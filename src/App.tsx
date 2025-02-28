import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [transcript, setTranscript] = useState("");
  const [isRecording, setIsRecording] = useState(false);

  async function startRecording() {
    setIsRecording(true);
    try {
      const result = await invoke("start_recording");
      setTranscript(result as string);
    } catch (error) {
      console.error("Erro ao capturar áudio:", error);
    } finally {
      setIsRecording(false);
    }
  }

  return (
    <div>
      <h1>Reconhecimento de Fala com Vosk</h1>
      <button onClick={startRecording} disabled={isRecording}>
        {isRecording ? "Gravando..." : "Iniciar Gravação"}
      </button>
      <div>
        <strong>Você disse:</strong> {transcript}
      </div>
    </div>
  );
}

export default App;