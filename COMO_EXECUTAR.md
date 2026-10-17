# Como executar a API e o site

## Requisitos

- Docker Desktop com Docker Compose (forma mais simples); ou Rust/Cargo, Node.js/npm e PostgreSQL para executar localmente.
- Clone o repositório e rode os comandos a partir da pasta raiz.

## Configuração

Crie o arquivo `.env` na raiz usando o exemplo:

```powershell
Copy-Item .env.example .env
```

A configuração padrão do Compose sobe PostgreSQL, API e site. Altere as senhas e os segredos antes de expor os serviços fora da sua máquina. Para autenticação do site, defina `AUTH_SECRET` com um valor aleatório forte. Não compartilhe nem versione `.env`.

## Executar tudo com Docker

Na raiz do projeto:

```powershell
docker compose up --build
```

Para executar em segundo plano:

```powershell
docker compose up --build -d
```

Endereços locais:

- Site: http://localhost:3000
- API: http://localhost:8080
- Verificação da API: http://localhost:8080/health

Ver logs: `docker compose logs -f api web db`. Parar os serviços: `docker compose down`. O comando `docker compose down -v` também apaga o volume local do banco.

## Executar localmente sem Docker

É necessário ter PostgreSQL ativo e configurar `DATABASE_URL` e `AUTH_SECRET` no ambiente usado pelo site. Instale as dependências do site e prepare o schema do banco:

```powershell
npm --prefix website install
npx --prefix website drizzle-kit push
```

Inicie a API em um terminal, na raiz:

```powershell
cargo run -p l96a1-api
```

Por padrão, a API escuta na porta `8080`. Para alterar, defina `PORT`; para ajustar os logs, use `RUST_LOG=info`.

Inicie o site em outro terminal:

```powershell
npm --prefix website run dev
```

O site fica em http://localhost:3000. Na execução local, configure `L96A1_API_URL=http://127.0.0.1:8080` para que o site encontre a API.

## Build de produção do site

```powershell
npm --prefix website run build
npm --prefix website run start
```

Confira `docker-compose.yml`, `.env.example` e os Dockerfiles para outras variáveis e opções de implantação.
