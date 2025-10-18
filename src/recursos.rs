// recursos.rs
use std::sync::Arc;
use tokio::sync::{Semaphore, OwnedSemaphorePermit, Mutex};
use tokio::time::{Duration, timeout, sleep, Instant};

// ---------------- Funções Utilitárias ----------------

/// Simula o uso de um recurso por um determinado tempo (com log)
pub async fn usar_recurso(nome: &str, duracao: u64) {
    println!("🔧 Utilizando recurso: {} por {} segundos...", nome, duracao);
    sleep(Duration::from_secs(duracao)).await;
    println!("🏁 Liberação do recurso: {}", nome);
}

/// Função auxiliar para pausar a execução entre etapas (para logs visíveis)
#[allow(dead_code)]
pub async fn pausa(segundos: u64) {
    sleep(Duration::from_secs(segundos)).await;
}

// ---------------- Estruturas de Rastreamento de Uso ----------------

#[derive(Debug, Clone)]
pub struct EventoUso {
    pub nome_paciente: String,
    pub inicio: f64,
    pub fim: f64,
    pub instancia_id: usize,
}

pub struct HistoricoUso {
    pub inicio_simulacao: Instant,
    pub medico: Mutex<Vec<EventoUso>>,
    pub sala: Mutex<Vec<EventoUso>>,
    pub leito: Mutex<Vec<EventoUso>>,
    pub exame: Mutex<Vec<EventoUso>>, // NOVO: Histórico para Exames
}

impl HistoricoUso {
    pub fn new() -> Self {
        Self {
            inicio_simulacao: Instant::now(),
            medico: Mutex::new(vec![]),
            sala: Mutex::new(vec![]),
            leito: Mutex::new(vec![]),
            exame: Mutex::new(vec![]), // NOVO: Inicialização do histórico
        }
    }

    // O método agora recebe o Mutex dos slots para atualização
    pub async fn registrar_inicio(&self, nome: &str, slots_mutex: &Arc<Mutex<Vec<Option<String>>>>, historico: &Mutex<Vec<EventoUso>>) -> Option<usize> {
        let now = self.inicio_simulacao.elapsed().as_secs_f64();
        let mut slots = slots_mutex.lock().await;

        for (i, slot) in slots.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(nome.to_string());
                
                historico.lock().await.push(EventoUso {
                    nome_paciente: nome.to_string(),
                    inicio: now,
                    fim: 0.0,
                    instancia_id: i + 1,
                });
                return Some(i + 1);
            }
        }
        None
    }

    // O método agora recebe o Mutex dos slots para atualização
    pub async fn registrar_fim(&self, nome: &str, slots_mutex: &Arc<Mutex<Vec<Option<String>>>>, historico: &Mutex<Vec<EventoUso>>) {
        let now = self.inicio_simulacao.elapsed().as_secs_f64();
        let mut hist_lock = historico.lock().await;
        
        // Atualiza o histórico de uso (registro de tempo final)
        if let Some(evento) = hist_lock.iter_mut().rev().find(|e| e.nome_paciente == nome && e.fim == 0.0) {
            evento.fim = now;
        }

        // Libera o slot de uso
        let mut slots = slots_mutex.lock().await;
        for slot in slots.iter_mut() {
            if let Some(paciente_nome) = slot {
                if paciente_nome == nome {
                    *slot = None; 
                    return;
                }
            }
        }
    }
}

// ---------------- ESTRUTURA RECURSOS (com lógica de Deadlock) ----------------

pub struct Recursos {
    pub medicos: Arc<Semaphore>,
    pub salas_cirurgia: Arc<Semaphore>,
    pub leitos: Arc<Semaphore>,
    pub equipamentos_exames: Arc<Semaphore>, // NOVO: Equipamentos de Exame

    // Slots de uso que a GUI irá ler
    pub slots_medicos: Arc<Mutex<Vec<Option<String>>>>,
    pub slots_salas: Arc<Mutex<Vec<Option<String>>>>,
    pub slots_leitos: Arc<Mutex<Vec<Option<String>>>>,
    pub slots_exames: Arc<Mutex<Vec<Option<String>>>>, // NOVO: Slots de Exames

    pub fila_medicos: Arc<Mutex<Vec<String>>>,
    pub fila_salas: Arc<Mutex<Vec<String>>>,
    pub fila_leitos: Arc<Mutex<Vec<String>>>,
    pub fila_exames: Arc<Mutex<Vec<String>>>, // NOVO: Fila de Exames

    pub deadlock_medicos: Arc<Mutex<Vec<String>>>,
    pub deadlock_salas: Arc<Mutex<Vec<String>>>,
    pub deadlock_leitos: Arc<Mutex<Vec<String>>>,
    pub deadlock_exames: Arc<Mutex<Vec<String>>>, // NOVO: Deadlock de Exames
    pub historico_uso: Arc<HistoricoUso>, 
}

impl Recursos {
    pub fn novo(qtd_medicos: usize, qtd_salas: usize, qtd_leitos: usize, qtd_exames: usize) -> Self {
        Self {
            medicos: Arc::new(Semaphore::new(qtd_medicos)),
            salas_cirurgia: Arc::new(Semaphore::new(qtd_salas)),
            leitos: Arc::new(Semaphore::new(qtd_leitos)),
            equipamentos_exames: Arc::new(Semaphore::new(qtd_exames)), // NOVO: Inicialização do semáforo

            // Inicialização dos slots
            slots_medicos: Arc::new(Mutex::new(vec![None; qtd_medicos])),
            slots_salas: Arc::new(Mutex::new(vec![None; qtd_salas])),
            slots_leitos: Arc::new(Mutex::new(vec![None; qtd_leitos])),
            slots_exames: Arc::new(Mutex::new(vec![None; qtd_exames])), // NOVO: Inicialização dos slots

            fila_medicos: Arc::new(Mutex::new(vec![])),
            fila_salas: Arc::new(Mutex::new(vec![])),
            fila_leitos: Arc::new(Mutex::new(vec![])),
            fila_exames: Arc::new(Mutex::new(vec![])), // NOVO: Inicialização da fila

            deadlock_medicos: Arc::new(Mutex::new(vec![])),
            deadlock_salas: Arc::new(Mutex::new(vec![])),
            deadlock_leitos: Arc::new(Mutex::new(vec![])),
            deadlock_exames: Arc::new(Mutex::new(vec![])), // NOVO: Inicialização do deadlock
            historico_uso: Arc::new(HistoricoUso::new()),
        }
    }

    /// Função de reserva que usa o novo Mutex dos slots e garante exclusividade por paciente.
    async fn reservar_recurso(
        fila: &Arc<Mutex<Vec<String>>>,
        recurso_sem: Arc<Semaphore>,
        deadlock: &Arc<Mutex<Vec<String>>>,
        slots: &Arc<Mutex<Vec<Option<String>>>>, 
        historico_uso: &Arc<HistoricoUso>,
        historico_eventos: &Mutex<Vec<EventoUso>>,
        nome: String,
        timeout_alerta_secs: u64,
    ) -> Result<OwnedSemaphorePermit, String> { // <-- Retorna Result para indicar falha
        
        // ---------------- GARANTIA DE EXCLUSIVIDADE ----------------
        // Verifica se o paciente já está ocupando algum slot deste tipo de recurso.
        {
            let slots_lock = slots.lock().await;
            if slots_lock.iter().any(|slot| slot.as_ref() == Some(&nome)) {
                // Se o nome do paciente já estiver em algum slot, a reserva é negada.
                return Err(format!("Paciente {} já está reservando um recurso deste tipo!", nome));
            }
        } // O lock 'slots_lock' é liberado aqui, antes de qualquer await longo.
        // ---------------- FIM GARANTIA DE EXCLUSIVIDADE ----------------

        let start_time = Instant::now();
        
        let mut f = fila.lock().await; 
        if !f.contains(&nome) {
            f.push(nome.clone());
        }
        drop(f);

        loop {
            let acquire_future = recurso_sem.clone().acquire_owned();

            let attempt = timeout(
                Duration::from_millis(500), 
                acquire_future
            ).await;

            match attempt {
                Ok(permit) => {
                    // SUCESSO!
                    fila.lock().await.retain(|n| n != &nome);
                    deadlock.lock().await.retain(|n| n != &nome);
                    
                    // REGISTRA O USO DO SLOT AQUI
                    historico_uso.registrar_inicio(&nome, slots, historico_eventos).await;
                    
                    return Ok(permit.unwrap()); // Retorno de sucesso com o OwnedSemaphorePermit
                }
                Err(_) => {
                    // TIMEOUT: Continua tentando.
                    if start_time.elapsed() >= Duration::from_secs(timeout_alerta_secs) {
                        let mut dl = deadlock.lock().await;
                        if !dl.contains(&nome) {
                            dl.push(nome.clone()); 
                        }
                    }
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }
    
    
    // ---------------- Lógica de Preempção (para o Monitor usar) ----------------

    /// Remove um paciente das filas de espera e de deadlock. 
    pub fn preempcao_paciente(&self, nome_paciente: &str) {
        // Limpa Filas e Deadlocks
        self.fila_medicos.blocking_lock().retain(|n| n != nome_paciente);
        self.deadlock_medicos.blocking_lock().retain(|n| n != nome_paciente);
        self.fila_salas.blocking_lock().retain(|n| n != nome_paciente);
        self.deadlock_salas.blocking_lock().retain(|n| n != nome_paciente);
        self.fila_leitos.blocking_lock().retain(|n| n != nome_paciente);
        self.deadlock_leitos.blocking_lock().retain(|n| n != nome_paciente);
        self.fila_exames.blocking_lock().retain(|n| n != nome_paciente); // NOVO
        self.deadlock_exames.blocking_lock().retain(|n| n != nome_paciente); // NOVO
    }
    
    // Funções auxiliares de liberação do slot (chamadas pelo código do Paciente após o uso)
    
    pub async fn liberar_medico_slot(&self, nome: &str) {
        self.historico_uso.registrar_fim(nome, &self.slots_medicos, &self.historico_uso.medico).await;
    }

    pub async fn liberar_sala_slot(&self, nome: &str) {
        self.historico_uso.registrar_fim(nome, &self.slots_salas, &self.historico_uso.sala).await;
    }

    pub async fn liberar_leito_slot(&self, nome: &str) {
        self.historico_uso.registrar_fim(nome, &self.slots_leitos, &self.historico_uso.leito).await;
    }

    pub async fn liberar_exame_slot(&self, nome: &str) { // NOVO
        self.historico_uso.registrar_fim(nome, &self.slots_exames, &self.historico_uso.exame).await;
    }


    // ---------------- Funções públicas de reserva (Atualizadas) ----------------
    
    pub async fn reservar_medico(&self, nome: String) -> Result<OwnedSemaphorePermit, String> { 
        Self::reservar_recurso(
            &self.fila_medicos,
            self.medicos.clone(),
            &self.deadlock_medicos,
            &self.slots_medicos, 
            &self.historico_uso, 
            &self.historico_uso.medico, 
            nome,
            10,
        ).await
    }

    pub async fn reservar_sala(&self, nome: String) -> Result<OwnedSemaphorePermit, String> { 
        Self::reservar_recurso(
            &self.fila_salas,
            self.salas_cirurgia.clone(),
            &self.deadlock_salas,
            &self.slots_salas, 
            &self.historico_uso, 
            &self.historico_uso.sala, 
            nome,
            10,
        ).await
    }

    pub async fn reservar_leito(&self, nome: String) -> Result<OwnedSemaphorePermit, String> { 
        Self::reservar_recurso(
            &self.fila_leitos,
            self.leitos.clone(),
            &self.deadlock_leitos,
            &self.slots_leitos, 
            &self.historico_uso, 
            &self.historico_uso.leito, 
            nome,
            10,
        ).await
    }

    pub async fn reservar_exame(&self, nome: String) -> Result<OwnedSemaphorePermit, String> { // NOVO
        Self::reservar_recurso(
            &self.fila_exames,
            self.equipamentos_exames.clone(),
            &self.deadlock_exames,
            &self.slots_exames, 
            &self.historico_uso, 
            &self.historico_uso.exame, 
            nome,
            10,
        ).await
    }
}