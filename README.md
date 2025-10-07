# SO_SIMULATOR
🔹 Instalação do Rust no Windows
=======================================================================
Baixe o instalador oficial:
=======================================================================
👉 https://win.rustup.rs

  Ele baixa o rustup-init.exe.
  
  Execute o instalador (pode aceitar a configuração padrão).
  
  Isso instala rustc, cargo e rustup.

=======================================================================
Depois de instalado, feche e reabra o PowerShell/VS Code e digite:
=======================================================================
  rustc --version
  cargo --version
Se responder com a versão, tudo está pronto. ✅

=======================================================================
Verificação final
=======================================================================

Digite:

  cargo new hello_rust
  cd hello_rust
  cargo run


Isso deve criar um projeto Rust, compilar e rodar o clássico Hello, world!, o arquivo main esta em hello_rust/src/main.rs.

=======================================================================
Como executar
=======================================================================

No terminal do projeto:

  cargo run

=======================================================================
📋 Saída esperada:
=======================================================================

Atendendo paciente: João (45) - Condição: Dor no peito
Atendendo paciente: Maria (32) - Condição: Fratura no braço

=======================================================================
🔹 Extensões principais
=======================================================================

Rust Analyzer (mais importante ✅)

  Fornece intellisense, autocompletar, navegação no código, diagnósticos e muito mais.

CodeLLDB

  Necessário se o senhor quiser depurar (debug) código Rust dentro do VS Code.

tokio

  cargo add tokio --features full

=======================================================================
ANOTAÇÕES
=======================================================================
 ainda nao consgui testar, mas tem o basico do funcionar e trabalharmos em cima dele;
 pedi a base pro chat; agora vou tentar estudar a sintaxe, para ver se entendo melhor.
 
(ta dando erro, o antivirus nao deixa passar, classifica como comportamento generico de trojan)

  rustup toolchain install stable-x86_64-pc-windows-gnu
  rustup default stable-x86_64-pc-windows-gnu
  cargo clean
  cargo run ( o erro aparece aqui, da nao permitido);



