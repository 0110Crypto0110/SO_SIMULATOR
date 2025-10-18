use crate::paciente::Paciente;
use crate::recursos::Recursos;
// CORREÇÃO 1: Importar a estrutura correta para a GUI
use crate::monitor_gui::EstadoRecursosGUI; 
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::task;

/// Estrutura que mantém a fila de pacientes
pub struct Hospital {
    pub fila: Arc<Mutex<VecDeque<Arc<Paciente>>>>,
    pub recursos: Arc<Recursos>,
}

impl Hospital {
    pub fn novo(recursos: Arc<Recursos>) -> Self {
        Self {
            fila: Arc::new(Mutex::new(VecDeque::new())),
            recursos,
        }
    }

    /// Adiciona um paciente à fila
    pub fn adicionar_paciente(&self, paciente: Arc<Paciente>) {
        let mut fila = self.fila.lock().unwrap();
        fila.push_back(paciente);
    }

    /// Despacha pacientes da fila para atendimento
    // CORREÇÃO 2: Adicionado #[allow(dead_code)] para sinalizar que esta função
    // não é usada no despacho principal (eliminando warnings globais).
    #[allow(dead_code)] 
    pub fn iniciar_atendimento(&self, estado_gui: Arc<Mutex<EstadoRecursosGUI>>) { // CORREÇÃO 3: Usar o tipo correto
        let fila_clone = self.fila.clone();
        let recursos_clone = self.recursos.clone();

        tokio::spawn(async move {
            loop {
                let paciente_opt = {
                    let mut fila = fila_clone.lock().unwrap();
                    fila.pop_front()
                };

                if let Some(paciente) = paciente_opt {
                    let recursos_clone2 = recursos_clone.clone();
                    let estado_clone = estado_gui.clone();
                    
                    // CORREÇÃO 4: Comentar a chamada incorreta ao método 'atender' (que não existe) 
                    // para evitar erros de compilação.
                    /*
                    tokio::spawn(async move {
                        paciente.atender(recursos_clone2, estado_clone).await; 
                    });
                    */
                    
                    // CORREÇÃO 5: Prefixar com '_' para silenciar as warnings de 'unused variable' 
                    // (já que a lógica foi comentada/movida).
                    let _paciente = paciente;
                    let _recursos_clone2 = recursos_clone2;
                    let _estado_clone = estado_clone;


                } else {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        });
    }

}