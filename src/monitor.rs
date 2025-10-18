// monitor.rs
use std::sync::Arc;
use tokio::time::{sleep, Duration, Instant};
use tokio::sync::Mutex; 
use crate::recursos::Recursos;
use crate::monitor_gui::EstadoRecursosGUI;
use crate::HistoricoRecursos;

// Definição auxiliar para o tipo de logs (AGORA USANDO tokio::sync::Mutex)
type Logs = Arc<Mutex<Vec<String>>>;

// Definições auxiliares para estado e histórico (usando tokio::sync::Mutex)
type EstadoGUI = Arc<Mutex<EstadoRecursosGUI>>;
type Historico = Arc<Mutex<HistoricoRecursos>>;


/// Monitora continuamente os recursos, atualiza a GUI e trata deadlocks.
pub async fn iniciar_monitor(
    recursos: Arc<Recursos>,
    // CORREÇÃO 1: Mudar para tokio::sync::Mutex
    estado_gui: EstadoGUI, 
    // CORREÇÃO 1: Mudar para tokio::sync::Mutex
    historico: Historico, 
    logs: Logs, // Arc<tokio::sync::Mutex<Vec<String>>>
) {
    let mut ultimo_log = Instant::now();

    loop {
        // Bloco para garantir que os locks sejam liberados após o uso.
        
        // Usa .lock().await para os Mutexes do tokio
        let mut estado = estado_gui.lock().await;
        // Usa .lock().await para os Mutexes do tokio
        let mut historico_lock = historico.lock().await; 
        
        let mut logs_lock = logs.lock().await; 
        
        // Record resource snapshot
        historico_lock.registrar(&recursos);
        
        // CORREÇÃO APLICADA: Substituído .lock().unwrap() por .lock().await
        // A função é ASYNC, e os campos deadlock_... são tokio::sync::Mutex
        let deadlock_m = recursos.deadlock_medicos.lock().await.clone();
        let deadlock_s = recursos.deadlock_salas.lock().await.clone();
        let deadlock_l = recursos.deadlock_leitos.lock().await.clone();
        
        let deadlock_detectado = !deadlock_m.is_empty()
            || !deadlock_s.is_empty()
            || !deadlock_l.is_empty();

        if deadlock_detectado {
            // Lógica de Tratamento de Deadlock (Preempção)
            
            let mut vitima = None;
            
            // Escolhe a vítima: Pela ordem de detecção (a mais simples)
            if let Some(p) = deadlock_m.iter().next() {
                vitima = Some(p.clone());
            } else if let Some(p) = deadlock_s.iter().next() {
                vitima = Some(p.clone());
            } else if let Some(p) = deadlock_l.iter().next() {
                vitima = Some(p.clone());
            }
            
            // Se encontramos uma vítima, tentamos "contornar" o deadlock
            if let Some(nome_vitima) = vitima {
                // **AÇÃO DE RECUPERAÇÃO**
                recursos.preempcao_paciente(&nome_vitima);
                
                let log_msg_contorno = format!(
                    "♻️ Deadlock Contornado: Paciente {} foi ABORTADO/LIMPO (preempção) para quebrar o ciclo de espera.",
                    nome_vitima
                );
                
                // Registra o contorno
                println!("\n[J.A.R.V.I.S.] {}", log_msg_contorno);
                // Adiciona ao log da GUI
                estado.registrar_log(&log_msg_contorno); 
                logs_lock.push(log_msg_contorno);
            }
        }
        
        // Log de Status Periódico
        let medicos_disp = recursos.medicos.available_permits();
        let salas_disp = recursos.salas_cirurgia.available_permits();
        let leitos_disp = recursos.leitos.available_permits();

        if ultimo_log.elapsed().as_secs() >= 1 {
            let log_msg = format!(
                "🔹 Status: Médicos disp: {}/{}, Salas disp: {}/{}, Leitos disp: {}/{}",
                medicos_disp, estado.medicos_max,
                salas_disp, estado.salas_max,
                leitos_disp, estado.leitos_max,
            );
            
            println!("\n[J.A.R.V.I.S.] {}", log_msg);
            estado.registrar_log(&log_msg);
            logs_lock.push(log_msg);
            
            ultimo_log = Instant::now();
        }
        
        // Atualiza o estado da GUI
        estado.medicos = medicos_disp;
        estado.salas = salas_disp;
        estado.leitos = leitos_disp;
        
        // Os MutexGuards 'estado', 'historico_lock' e 'logs_lock' são liberados aqui 
        // ao sair do escopo, o que é seguro em tokio.
        
        // Intervalo de atualização (AWAIT, por isso precisamos do tokio::sync::Mutex para logs, estado e historico)
        sleep(Duration::from_millis(500)).await;
    }
}