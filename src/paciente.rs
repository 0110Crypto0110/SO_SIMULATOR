use crate::recursos::{Recursos, usar_recurso, pausa};
use crate::monitor_gui::EstadoRecursosGUI;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

// Nota: OwnedSemaphorePermit não é mais importado diretamente, 
// pois as permissões serão declaradas no escopo do match/if let.

#[allow(dead_code)] // Suppress unused field warnings
pub struct Paciente {
    pub nome: String,
    pub idade: u32,
    pub condicao: String,
    pub precisa_cirurgia: bool,
    pub prioridade: u8, // 0 = normal, 1 = crítico
}

impl Paciente {
    // ... (funções 'novo' e 'novo_com_prioridade' permanecem iguais)
    pub fn novo(nome: &str, idade: u32, condicao: &str, precisa_cirurgia: bool, prioridade: u8) -> Self {
        Paciente {
            nome: nome.to_string(),
            idade,
            condicao: condicao.to_string(),
            precisa_cirurgia,
            prioridade,
        }
    }

    pub fn novo_com_prioridade(nome: &str, idade: u32, condicao: &str, precisa_cirurgia: bool, prioridade: u8) -> Self {
        Self::novo(nome, idade, condicao, precisa_cirurgia, prioridade)
    }

    pub async fn atender_com_escala(
        &self,
        recursos: Arc<Recursos>,
        estado_gui: Arc<Mutex<EstadoRecursosGUI>>,
        escala_tempo: f64,
    ) -> bool {
        // Nova contagem de etapas:
        // Consulta (1) + Exames (1) + Cirurgia/Leito (1 ou 2)
        // Se cirurgia: 4 etapas (Consulta, Exame, Cirurgia, Leito)
        // Se não cirurgia: 3 etapas (Consulta, Exame, Leito Observação)
        let num_etapas = if self.precisa_cirurgia { 4.0 } else { 3.0 };
        let mut progresso = 0.0;
        let nome_paciente = self.nome.clone();

        pausa((0.5 * escala_tempo) as u64).await;
        let inicio_atendimento = Instant::now();

        // Variável para manter a permissão do médico, se necessário (Cirurgia)
        let mut medico_permit_op = None;
        
        // -------------------- ETAPA 1: Médico (Consulta/Avaliação) --------------------
        {
            let mut estado = estado_gui.lock().await;
            if self.prioridade == 1 {
                estado.fila_medicos.insert(0, nome_paciente.clone());
            } else {
                estado.fila_medicos.push(nome_paciente.clone());
            }
            estado.atualizar_progresso(&nome_paciente, progresso / num_etapas as f32);
            estado.registrar_log(&format!("🔹 {} entrou na fila de médicos", nome_paciente));
        }

        let medico_permit = match recursos.reservar_medico(nome_paciente.clone()).await {
            Ok(permit) => permit,
            Err(e) => {
                let mut estado = estado_gui.lock().await;
                estado.registrar_log(&format!("❌ {} Falha na reserva de médico (Exclusividade): {}", nome_paciente, e));
                return false;
            }
        };

        {
            let mut estado = estado_gui.lock().await;
            estado.fila_medicos.retain(|n| n != &nome_paciente);
            recursos.historico_uso.registrar_inicio(&nome_paciente, &recursos.slots_medicos, &recursos.historico_uso.medico).await;
            progresso += 1.0;
            estado.atualizar_progresso(&nome_paciente, progresso / num_etapas as f32);
            estado.registrar_log(&format!("✅ {} começou consulta com médico", nome_paciente));
        }

        usar_recurso(&format!("Consulta - {}", nome_paciente), (3.0 * escala_tempo) as u64).await;

        // Se precisa de cirurgia, guarda a permissão do médico para mantê-lo na próxima etapa
        if self.precisa_cirurgia {
            medico_permit_op = Some(medico_permit);
            let mut estado = estado_gui.lock().await;
            estado.registrar_log(&format!("🩺 {} Manteve médico para cirurgia", nome_paciente));
        } else {
            // Se não precisa de cirurgia, libera o médico após a consulta
            recursos.liberar_medico_slot(&nome_paciente).await;
            drop(medico_permit);
            let mut estado = estado_gui.lock().await;
            estado.registrar_log(&format!("✅ {} liberou médico após consulta", nome_paciente));
        }
        
        // -------------------- ETAPA 2: Exames (Obrigatório) --------------------
        {
            let mut estado = estado_gui.lock().await;
            if self.prioridade == 1 {
                estado.fila_exames.insert(0, nome_paciente.clone());
            } else {
                estado.fila_exames.push(nome_paciente.clone());
            }
            estado.atualizar_progresso(&nome_paciente, progresso / num_etapas as f32);
            estado.registrar_log(&format!("🔹 {} entrou na fila de exames", nome_paciente));
        }

        let exame_permit = match recursos.reservar_exame(nome_paciente.clone()).await {
            Ok(permit) => permit,
            Err(e) => {
                let mut estado = estado_gui.lock().await;
                estado.registrar_log(&format!("❌ {} Falha na reserva de exame (Exclusividade): {}", nome_paciente, e));
                return false;
            }
        };

        {
            let mut estado = estado_gui.lock().await;
            estado.fila_exames.retain(|n| n != &nome_paciente);
            recursos.historico_uso.registrar_inicio(&nome_paciente, &recursos.slots_exames, &recursos.historico_uso.exame).await;
            progresso += 1.0;
            estado.atualizar_progresso(&nome_paciente, progresso / num_etapas as f32);
            estado.registrar_log(&format!("🔬 {} começou a fazer exames", nome_paciente));
        }

        usar_recurso(&format!("Exame - {}", nome_paciente), (2.0 * escala_tempo) as u64).await;

        recursos.liberar_exame_slot(&nome_paciente).await;
        drop(exame_permit);
        {
            let mut estado = estado_gui.lock().await;
            estado.registrar_log(&format!("✅ {} terminou exames e liberou equipamento", nome_paciente));
        }

        // -------------------- ETAPA 3 e 4: Tratamento (Cirurgia OU Observação/Leito) --------------------

        if self.precisa_cirurgia {
            // --- ETAPA 3: Sala de Cirurgia (com Médico) ---
            let inicio_sala = Instant::now();
            {
                let mut estado = estado_gui.lock().await;
                if self.prioridade == 1 {
                    estado.fila_salas.insert(0, nome_paciente.clone());
                } else {
                    estado.fila_salas.push(nome_paciente.clone());
                }
                estado.atualizar_progresso(&nome_paciente, progresso / num_etapas as f32);
                estado.registrar_log(&format!("🔹 {} entrou na fila de salas", nome_paciente));
            }

            let sala_permit = match recursos.reservar_sala(nome_paciente.clone()).await {
                Ok(permit) => permit,
                Err(e) => {
                    let mut estado = estado_gui.lock().await;
                    estado.registrar_log(&format!("❌ {} Falha na reserva de sala (Exclusividade): {}", nome_paciente, e));
                    return false;
                }
            };

            {
                let espera_sala = inicio_sala.elapsed();
                let mut estado = estado_gui.lock().await;
                estado.fila_salas.retain(|n| n != &nome_paciente);
                recursos.historico_uso.registrar_inicio(&nome_paciente, &recursos.slots_salas, &recursos.historico_uso.sala).await;
                progresso += 1.0;
                estado.atualizar_progresso(&nome_paciente, progresso / num_etapas as f32);
                estado.registrar_atendimento(&nome_paciente, espera_sala);
                estado.registrar_log(&format!("🔪 {} começou cirurgia (Médico + Sala)", nome_paciente));
            }

            usar_recurso(&format!("Cirurgia - {}", nome_paciente), (4.0 * escala_tempo) as u64).await;

            // Liberação da Sala e do Médico após a cirurgia
            recursos.liberar_sala_slot(&nome_paciente).await;
            drop(sala_permit);
            
            if let Some(medico) = medico_permit_op.take() {
                recursos.liberar_medico_slot(&nome_paciente).await;
                drop(medico);
            }
            {
                let mut estado = estado_gui.lock().await;
                estado.registrar_log(&format!("✅ {} terminou cirurgia, liberou Sala e Médico", nome_paciente));
            }

            // --- ETAPA 4: Leito pós-cirurgia (Recuperação) ---
            let inicio_leito = Instant::now();
            {
                let mut estado = estado_gui.lock().await;
                if self.prioridade == 1 {
                    estado.fila_leitos.insert(0, nome_paciente.clone());
                } else {
                    estado.fila_leitos.push(nome_paciente.clone());
                }
                estado.atualizar_progresso(&nome_paciente, progresso / num_etapas as f32);
                estado.registrar_log(&format!("🔹 {} entrou na fila de leitos (Recuperação)", nome_paciente));
            }

            let leito_permit = match recursos.reservar_leito(nome_paciente.clone()).await {
                Ok(permit) => permit,
                Err(e) => {
                    let mut estado = estado_gui.lock().await;
                    estado.registrar_log(&format!("❌ {} Falha na reserva de leito (Exclusividade): {}", nome_paciente, e));
                    return false;
                }
            };

            {
                let espera_leito = inicio_leito.elapsed();
                let mut estado = estado_gui.lock().await;
                estado.fila_leitos.retain(|n| n != &nome_paciente);
                recursos.historico_uso.registrar_inicio(&nome_paciente, &recursos.slots_leitos, &recursos.historico_uso.leito).await;
                progresso += 1.0;
                estado.atualizar_progresso(&nome_paciente, progresso / num_etapas as f32);
                estado.registrar_atendimento(&nome_paciente, espera_leito);
                estado.registrar_log(&format!("🛌 {} começou recuperação em leito", nome_paciente));
            }

            usar_recurso(&format!("Leito - {}", nome_paciente), (3.0 * escala_tempo) as u64).await;

            recursos.liberar_leito_slot(&nome_paciente).await;
            drop(leito_permit);
        } else {
            // --- ETAPA 3: Leito (Observação/Sem Cirurgia) ---
            let inicio_leito = Instant::now();
            {
                let mut estado = estado_gui.lock().await;
                if self.prioridade == 1 {
                    estado.fila_leitos.insert(0, nome_paciente.clone());
                } else {
                    estado.fila_leitos.push(nome_paciente.clone());
                }
                estado.atualizar_progresso(&nome_paciente, progresso / num_etapas as f32);
                estado.registrar_log(&format!("🔹 {} entrou na fila de leitos (Observação)", nome_paciente));
            }

            let leito_permit = match recursos.reservar_leito(nome_paciente.clone()).await {
                Ok(permit) => permit,
                Err(e) => {
                    let mut estado = estado_gui.lock().await;
                    estado.registrar_log(&format!("❌ {} Falha na reserva de leito (Exclusividade): {}", nome_paciente, e));
                    return false;
                }
            };
            
            {
                let espera_leito = inicio_leito.elapsed();
                let mut estado = estado_gui.lock().await;
                estado.fila_leitos.retain(|n| n != &nome_paciente);
                recursos.historico_uso.registrar_inicio(&nome_paciente, &recursos.slots_leitos, &recursos.historico_uso.leito).await;
                progresso += 1.0;
                estado.atualizar_progresso(&nome_paciente, progresso / num_etapas as f32);
                estado.registrar_atendimento(&nome_paciente, espera_leito);
                estado.registrar_log(&format!("🛌 {} começou observação em leito", nome_paciente));
            }

            usar_recurso(&format!("Leito - {}", nome_paciente), (3.0 * escala_tempo) as u64).await;

            recursos.liberar_leito_slot(&nome_paciente).await;
            drop(leito_permit);
        }

        // -------------------- ETAPA FINAL: Saída --------------------

        let espera_total = inicio_atendimento.elapsed();
        {
            let mut estado = estado_gui.lock().await;
            estado.registrar_atendimento(&nome_paciente, espera_total);
            estado.registrar_log(&format!("🎉 {} concluiu atendimento.", nome_paciente));
        }

        pausa((1.0 * escala_tempo) as u64).await;
        {
            let mut estado = estado_gui.lock().await;
            estado.atualizar_progresso(&nome_paciente, 1.0);
            estado.registrar_log(&format!("✅ {} finalizou progresso", nome_paciente));
        }

        // Se a função chegou a este ponto, o atendimento foi bem-sucedido.
        true
    }
}
