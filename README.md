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


