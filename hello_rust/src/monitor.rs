use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration, Instant};
use crate::recursos::Recursos;
use crate::monitor_gui::EstadoRecursosGUI;

/// Monitora continuamente os recursos do hospital e atualiza o estado da GUI.
/// Detecta deadlocks e registra logs periódicos no console.
pub async fn iniciar_monitor(recursos: Arc<Recursos>, estado_gui: Arc<Mutex<EstadoRecursosGUI>>) {
    tokio::spawn(async move {
        let mut ultimo_log = Instant::now();

        loop {
            {
                // Bloqueia o estado momentaneamente para atualização
                let mut estado = estado_gui.lock().unwrap();

                // Atualiza o estado do GUI com a situação atual dos recursos
                estado.atualizar_estado(&recursos);

                // 🔹 Verificação de deadlocks simples
                let deadlock_detectado = !estado.deadlock_medicos.is_empty()
                    || !estado.deadlock_salas.is_empty()
                    || !estado.deadlock_leitos.is_empty();

                if deadlock_detectado {
                    println!(
                        "[J.A.R.V.I.S.] ⚠️ Alerta: Deadlock detectado!\n\
                         Médicos: {:?}\n\
                         Salas: {:?}\n\
                         Leitos: {:?}",
                        estado.deadlock_medicos, estado.deadlock_salas, estado.deadlock_leitos
                    );
                }

                // 🔹 Log periódico da situação dos recursos (a cada 3 segundos)
                if ultimo_log.elapsed().as_secs() >= 3 {
                    println!(
                        "\n[J.A.R.V.I.S.] Status dos Recursos 🩺\n\
                         Médicos disponíveis: {}/{}\n\
                         Salas de cirurgia disponíveis: {}/{}\n\
                         Leitos disponíveis: {}/{}\n\
                         Fila Médicos: {:?}\n\
                         Fila Salas: {:?}\n\
                         Fila Leitos: {:?}\n",
                        estado.medicos,
                        estado.medicos_max,
                        estado.salas,
                        estado.salas_max,
                        estado.leitos,
                        estado.leitos_max,
                        estado.fila_medicos,
                        estado.fila_salas,
                        estado.fila_leitos
                    );
                    ultimo_log = Instant::now();
                }
                // 🔓 Mutex é liberado aqui antes do await
            }

            // Intervalo de atualização
            sleep(Duration::from_millis(500)).await;
        }
    });
}

