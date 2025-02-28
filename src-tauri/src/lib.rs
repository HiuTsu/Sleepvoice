use vosk::{Model, Recognizer};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_recording])
        .run(tauri::generate_context!())
        .expect("Erro ao rodar o aplicativo Tauri");
}

#[tauri::command]
fn start_recording() -> String {
    // Caminho para o modelo Vosk
    let model_path = "src-tauri/models/vosk-model-small-pt-0.3".to_string();
    let model = Model::new(&model_path).expect("Não foi possível carregar o modelo");

    // Taxa de amostragem do áudio (16 kHz é comum para modelos Vosk)
    let sample_rate = 32000.0;
    let mut recognizer = Recognizer::new(&model, sample_rate).expect("Não foi possível criar o reconhecedor");

    // Configuração da captura de áudio com cpal
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("Nenhum dispositivo de entrada encontrado");
    let config = input_device.default_input_config().expect("Falha ao obter configuração de áudio");

    // Buffer para armazenar os dados de áudio
    let audio_data = Arc::new(Mutex::new(Vec::new()));

    // Função de callback para captura de áudio
    let audio_data_clone = audio_data.clone();
    let stream = input_device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut audio_data = audio_data_clone.lock().unwrap();

            // Converte f32 para i16 e armazena no buffer
            for &sample in data {
                let sample_i16 = (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                audio_data.push(sample_i16);
            }
        },
        move |err| {
            eprintln!("Erro na captura de áudio: {}", err);
        },
    ).expect("Falha ao criar stream de áudio");

    // Inicia a captura de áudio
    stream.play().expect("Falha ao iniciar a captura de áudio");

    // Grava por 1 segundo
    thread::sleep(Duration::from_secs(1));

    // Para a captura de áudio
    drop(stream);

    // Processa o áudio capturado com o Vosk
    let audio_data = audio_data.lock().unwrap();

    // Em vez de mover, agora usamos a referência
    recognizer.accept_waveform(&audio_data).expect("Falha ao processar o áudio");

    // Obtém o resultado final
    let result = recognizer.final_result().single().expect("Falha ao obter o resultado final");
    result.text.to_string()
}
