Solana Arbitragem
Este projeto é um bot de arbitragem em modo simulação segura para a blockchain Solana, escrito em Rust. Ele consulta a API da Jupiter para avaliar, em loop, se existe oportunidade de lucro ao fazer um ciclo SOL → USDC → SOL, levando em conta um custo mínimo estimado de taxas on-chain.

Como o bot funciona
O bot usa reqwest para chamar a API pública da Jupiter (/swap/v1/quote) duas vezes por ciclo: primeira cotação de SOL para USDC e, em seguida, de USDC para SOL. Com base nesses dois valores, ele calcula a diferença entre o saldo inicial em SOL e o saldo final simulado após o round trip.

Em seguida, ele compara essa diferença com um custo estimado de rede de 80.000 lamports (fees base + prioridade) e só considera uma oportunidade “real” se o lucro bruto for maior que esse custo. Além disso, existe uma trava de segurança: se o lucro líquido for maior que 50% do investimento em uma única operação, o bot trata como possível bug/honeypot da API e ignora o sinal.

O bot não envia nenhuma transação on-chain no estado atual: ele só simula, loga e imprime as oportunidades detectadas.

Configuração atual (do código)
As configurações principais estão hardcoded em src/main.rs:

API da Jupiter: https://api.jup.ag/swap/v1/quote

Header x-api-key: valor definido em my_api_key no código

Mint de entrada (SOL): So11111111111111111111111111111111111111112

Mint de saída (USDC): EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v

Quantidade por operação: sol_inicial = 50_000_000u64 (0,05 SOL)

Slippage: slippageBps=50 (0,5%) em ambas as cotações

Custo estimado de rede: 80_000 lamports (break-even mínimo)

Trava de segurança de lucro: lucro_liquido > sol_inicial / 2 é tratado como suspeito e ignorado

Para a carteira, o código tenta carregar meu-bot.json via read_keypair_file; se não encontrar, cria um Keypair::new() apenas para ter uma public key para exibição. Como não há envio de transações, essa chave não é usada para gastar fundos no estado atual.

Fluxo de execução
Inicializa o cliente HTTP (reqwest::Client) aceitando certificados inválidos (danger_accept_invalid_certs(true)).

Tenta carregar a keypair de meu-bot.json; se falhar, gera uma carteira fake para exibir o pubkey.

Entra em um loop infinito (loop { ... }) com as etapas:

Cota SOL → USDC com o valor sol_inicial.

Lê o campo outAmount retornado e converte para u64 (USDC em unidades mínimas).

Cota USDC → SOL usando o outAmount da primeira cotação.

Lê o campo outAmount da segunda cotação (SOL final) e converte para u64.

Calcula diff = sol_final - sol_inicial e aplica as regras de custo e segurança.

Se for lucro considerado real, imprime informações detalhadas no terminal e grava uma linha em log_seguro.txt com o lucro líquido e timestamp.

Se não houver oportunidade, apenas imprime pontos no stdout para indicar que o bot está rodando.

Entre cada iteração completa, o bot dorme 2 segundos; após uma oportunidade real, dorme 10 segundos para não floodar o log.

Logs e relatórios
O bot escreve logs em um arquivo log_seguro.txt na raiz do projeto. Cada oportunidade real encontrada gera uma linha no formato:

text
REAL: Lucro Líquido <valor> lamports | Data: <SystemTime>
Atualmente não há escrita estruturada em relatorio.txt no código fornecido; esse arquivo existe no repositório, mas ainda não é atualizado por main.rs.

Requisitos
Para rodar o projeto, você precisa de:

Rust e Cargo instalados (veja Cargo.toml para detalhes de dependências).

Acesso à internet para consultar a API da Jupiter.

Opcional: um arquivo meu-bot.json com uma keypair Solana válida, caso queira mostrar sua própria carteira no log.

Instalação
Clone o repositório e faça o build:

bash
git clone https://github.com/Vinisilva0010/solana-arbitragem.git
cd solana-arbitragem
cargo build --release
Execução
Para iniciar o bot em modo simulação segura:

bash
cargo run --release
Antes de rodar, revise diretamente em src/main.rs:

O valor de sol_inicial que você quer simular.

O custo estimado de rede (custo_estimado_rede).

O my_api_key usado para acessar a API da Jupiter.

Lembre que, na versão atual, nenhuma transação é enviada para a rede Solana: tudo é simulado a partir das cotações da API.

Avisos de segurança
Mesmo em modo simulação, este tipo de projeto normalmente evolui para enviar transações reais, então é importante manter boas práticas desde o início:

Não faça commit de chaves privadas (como meu-bot.json) nem de API keys sensíveis.

Use variáveis de ambiente ou arquivos ignorados no .gitignore para credenciais.

Teste sempre com pequenas quantidades e entenda o comportamento da Jupiter e das taxas antes de arriscar fundos reais.
