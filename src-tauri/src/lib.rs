use tauri::Builder;
use tauri::generate_handler;
use vosk::{Model, Recognizer};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::Write;

// Exporte a função run
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(generate_handler![start_recording]) // Registre o comando
        .run(tauri::generate_context!())
        .expect("Erro ao rodar o aplicativo Tauri");
}

// Comando start_recording
#[tauri::command]
fn start_recording() -> String {
    // Caminho para o modelo Vosk
    let model_path = "C:\\Users\\Administrator\\Desktop\\Projetos\\Sleepvoice\\src-tauri\\models\\vosk-model-pt-fb";
    let model = Model::new(model_path).expect("Não foi possível carregar o modelo");

    // Configuração do dispositivo de áudio
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("Nenhum dispositivo de entrada encontrado");
    let config = input_device.default_input_config().expect("Falha ao obter configuração de áudio padrão");

    // Taxa de amostragem do áudio
    let sample_rate = config.sample_rate().0 as f32;
    println!("Taxa de amostragem do dispositivo: {} Hz", sample_rate);
    let mut recognizer = Recognizer::new(&model, sample_rate).expect("Não foi possível criar o reconhecedor");

    // Canal para transferir dados de áudio entre threads
    let (sender, receiver) = mpsc::sync_channel(8192); // Buffer maior para evitar bloqueios

    // Função de callback para captura de áudio
    let stream = input_device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut samples_i16 = Vec::with_capacity(data.len());
            for &sample in data {
                let sample_i16 = ((sample * i16::MAX as f32).round() as i16).saturating_mul(10); // Aumenta o ganho
                samples_i16.push(sample_i16);
            }
            if sender.send(samples_i16).is_err() {
                eprintln!("Falha ao enviar amostras de áudio");
            }
        },
        move |err| {
            eprintln!("Erro na captura de áudio: {}", err);
        },
    ).expect("Falha ao criar stream de áudio");

    // Inicia a captura de áudio
    stream.play().expect("Falha ao iniciar a captura de áudio");

    // Grava por 10 segundos
    let start_time = std::time::Instant::now();
    let mut audio_buffer = Vec::new();

    while start_time.elapsed() < Duration::from_secs(10) {
        // Coleta amostras em um buffer
        while let Ok(samples) = receiver.try_recv() {
            println!("Amostras de áudio recebidas: {}", samples.len());
            audio_buffer.extend(samples);
            if audio_buffer.len() >= 16000 { // Limita o tamanho do buffer (1 segundo de áudio)
                break;
            }
        }

        // Processa o buffer de áudio
        if !audio_buffer.is_empty() {
            match recognizer.accept_waveform(&audio_buffer) {
                Ok(_) => {
                    let partial_result = recognizer.partial_result();
                    println!("Resultado parcial: {:?}", partial_result);
                }
                Err(err) => {
                    eprintln!("Erro ao processar áudio: {:?}", err);
                }
            }
            audio_buffer.clear(); // Limpa o buffer após o processamento
        }

        thread::sleep(Duration::from_millis(10)); // Reduz o tempo de espera
    }

    // Para a captura de áudio
    drop(stream);

    // Obtém o resultado final
    let result = recognizer.final_result();
    println!("Resultado final: {:?}", result); // Log do resultado

    // Grava o áudio capturado em um arquivo para depuração
    let mut file = File::create("audio_capturado.raw").expect("Falha ao criar arquivo de áudio");

    // Converte o vetor de i16 para u8
    let audio_buffer_u8: Vec<u8> = audio_buffer
        .iter()
        .flat_map(|&sample| sample.to_le_bytes().to_vec())
        .collect();

    // Grava os dados convertidos no arquivo
    file.write_all(&audio_buffer_u8).expect("Falha ao gravar áudio no arquivo");

    // Retorna o texto reconhecido
    result.single().expect("Falha ao obter o resultado final").text.to_string()
}