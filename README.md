# My Cloud Messaging

Sistema de push notification self-hosted, similar ao Firebase Cloud Messaging (FCM). Implementado em Rust usando o protocolo Web Push (RFC 8030 + RFC 8291) com autenticação VAPID.

Funciona com todos os browsers modernos: Chrome, Firefox, Edge, Opera e Safari (iOS 16.4+).

## Como funciona

```
Seu servidor  ──POST──►  Push Service (Google/Mozilla/Apple)  ──►  Browser do usuário
              (VAPID + AES-128-GCM encrypted)
```

O browser do usuário se registra no seu servidor com um `endpoint` fornecido pelo browser. Quando você envia uma notificação, seu servidor faz um POST para esse endpoint — o push service do fabricante do browser entrega a mensagem. O conteúdo é criptografado ponta-a-ponta; o push service não consegue ler.

Não é necessário cadastro nem pagamento em nenhum serviço externo.

## Requisitos

- Docker e Docker Compose

## Configuração

```bash
cp .env.example .env
```

Edite o `.env` com suas configurações:

| Variável | Descrição | Obrigatório |
|---|---|---|
| `POSTGRES_PASSWORD` | Senha do banco de dados | Sim |
| `API_KEY` | Token para proteger os endpoints administrativos | Sim |
| `VAPID_CONTACT` | Email de contato (exigido pelo protocolo VAPID) | Não |
| `PORT` | Porta do servidor (padrão: 8080) | Não |
| `CLEANUP_INTERVAL_HOURS` | Intervalo de limpeza de subscriptions expiradas (padrão: 6) | Não |
| `NOTIFICATION_TTL_SECONDS` | Tempo de vida das notificações (padrão: 86400) | Não |

Gere uma API_KEY segura:
```bash
openssl rand -hex 32
```

## Iniciando

```bash
docker compose up -d
```

Na primeira execução, o servidor gera automaticamente um par de chaves VAPID e salva no banco. A chave pública fica disponível em `/vapid-public-key` e será usada no script JS do seu site.

Para rebuildar após mudanças no código:
```bash
docker compose build && docker compose up -d
```

Verificar se está rodando:
```bash
curl http://localhost:8080/health
```

```json
{ "status": "ok", "database": "ok", "version": "0.1.0" }
```

---

## API

### Endpoints públicos

Não exigem autenticação. Use nos endpoints que o browser acessa diretamente.

---

#### `GET /health`

Verifica se o servidor e o banco estão operacionais.

**Resposta:**
```json
{ "status": "ok", "database": "ok", "version": "0.1.0" }
```

---

#### `GET /vapid-public-key`

Retorna a chave pública VAPID. Use no script JS do seu site para chamar `pushManager.subscribe()`.

**Resposta:**
```json
{ "public_key": "BLroTZkQ_2aKAfxv7165mW..." }
```

---

#### `POST /subscriptions`

Registra um dispositivo. Chame este endpoint quando o usuário aceitar as notificações no browser.

**Body:**
```json
{
  "endpoint": "https://fcm.googleapis.com/fcm/send/...",
  "keys": {
    "p256dh": "BNcRdreALRFXTkOO...",
    "auth": "tBHItJI5svbpez7KI4CCfw=="
  },
  "tags": ["user_123", "premium"],
  "metadata": { "locale": "pt-BR" }
}
```

| Campo | Tipo | Descrição |
|---|---|---|
| `endpoint` | string | URL do push service (vem do browser) |
| `keys.p256dh` | string | Chave pública ECDH do browser (base64url) |
| `keys.auth` | string | Segredo de autenticação (base64url) |
| `tags` | array | Tags para filtrar envios (opcional) |
| `metadata` | object | Dados extras livres (opcional) |

**Resposta 201:**
```json
{ "id": "550e8400-e29b-41d4-a716-446655440000", "created_at": "2024-01-01T00:00:00Z" }
```

**Resposta 409** (endpoint já registrado):
```json
{ "error": "subscription_exists", "id": "550e8400-e29b-41d4-a716-446655440000" }
```

---

### Endpoints protegidos

Exigem o header `Authorization: Bearer <API_KEY>`.

---

#### `GET /subscriptions`

Lista todos os dispositivos registrados.

**Query params:**
- `tag` — filtra por tag
- `page` — página (padrão: 1)
- `per_page` — itens por página, máximo 100 (padrão: 50)

**Exemplo:**
```bash
curl http://localhost:8080/subscriptions?tag=premium \
  -H "Authorization: Bearer <API_KEY>"
```

---

#### `DELETE /subscriptions/{id}`

Remove um dispositivo pelo UUID.

**Resposta:** `204 No Content`

---

#### `POST /notifications/send`

Envia uma notificação para uma lista específica de dispositivos.

**Body:**
```json
{
  "subscription_ids": [
    "550e8400-e29b-41d4-a716-446655440000",
    "660e8400-e29b-41d4-a716-446655440001"
  ],
  "notification": {
    "title": "Nova mensagem",
    "body": "Você recebeu uma mensagem de João",
    "icon": "/icon-192.png",
    "url": "https://meusite.com/mensagens/42",
    "data": { "message_id": 42 }
  },
  "ttl": 86400
}
```

| Campo | Tipo | Descrição |
|---|---|---|
| `subscription_ids` | array | UUIDs dos dispositivos |
| `notification.title` | string | Título da notificação |
| `notification.body` | string | Corpo da notificação |
| `notification.icon` | string | URL do ícone (opcional) |
| `notification.url` | string | URL ao clicar na notificação (opcional) |
| `notification.data` | object | Dados extras para o service worker (opcional) |
| `ttl` | number | Tempo de vida em segundos (opcional, padrão: 86400) |

**Resposta:**
```json
{
  "total": 2,
  "sent": 2,
  "failed": 0,
  "gone": 0,
  "results": [
    { "subscription_id": "550e8400-...", "status": "sent" },
    { "subscription_id": "660e8400-...", "status": "sent" }
  ]
}
```

Status possíveis por dispositivo: `sent`, `failed`, `gone` (subscription expirada — removida automaticamente).

---

#### `POST /notifications/broadcast`

Envia para todos os dispositivos, ou para os que possuem determinadas tags.

**Body:**
```json
{
  "tags": ["premium"],
  "notification": {
    "title": "Promoção exclusiva",
    "body": "50% de desconto por tempo limitado"
  }
}
```

Deixe `tags` como array vazio ou omita para enviar para todos os dispositivos.

**Resposta:**
```json
{ "total": 150, "sent": 148, "failed": 0, "gone": 2 }
```

---

#### `GET /notifications/log`

Histórico dos envios de notificações.

**Query params:**
- `status` — filtra por status: `sent`, `failed`, `gone`, `pending`
- `page` — página (padrão: 1)
- `per_page` — itens por página, máximo 100 (padrão: 50)

---

## Integração no site (JavaScript)

O script do lado do browser precisa de um **service worker** para receber as notificações.

### 1. Crie o service worker (`sw.js`)

```javascript
self.addEventListener('push', event => {
  const data = event.data?.json() ?? {};
  event.waitUntil(
    self.registration.showNotification(data.title ?? 'Notificação', {
      body: data.body,
      icon: data.icon ?? '/icon-192.png',
      data: { url: data.url },
    })
  );
});

self.addEventListener('notificationclick', event => {
  event.notification.close();
  if (event.notification.data?.url) {
    event.waitUntil(clients.openWindow(event.notification.data.url));
  }
});
```

### 2. Registre no seu site

```javascript
const VAPID_PUBLIC_KEY = 'BLroTZkQ_2aKAfxv7165mW...'; // de GET /vapid-public-key
const API_BASE = 'https://meu-servidor.com';

async function enableNotifications() {
  if (!('serviceWorker' in navigator) || !('PushManager' in window)) {
    console.warn('Push não suportado neste browser');
    return;
  }

  const registration = await navigator.serviceWorker.register('/sw.js');
  await navigator.serviceWorker.ready;

  const permission = await Notification.requestPermission();
  if (permission !== 'granted') return;

  // Converte a chave pública de base64url para Uint8Array
  const key = Uint8Array.from(
    atob(VAPID_PUBLIC_KEY.replace(/-/g, '+').replace(/_/g, '/')),
    c => c.charCodeAt(0)
  );

  const subscription = await registration.pushManager.subscribe({
    userVisibleOnly: true,
    applicationServerKey: key,
  });

  // Envia para seu servidor
  await fetch(`${API_BASE}/subscriptions`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      endpoint: subscription.endpoint,
      keys: {
        p256dh: btoa(String.fromCharCode(...new Uint8Array(subscription.getKey('p256dh'))))
          .replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, ''),
        auth: btoa(String.fromCharCode(...new Uint8Array(subscription.getKey('auth'))))
          .replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, ''),
      },
      tags: ['minha-tag'],
    }),
  });

  console.log('Notificações ativadas!');
}
```

---

## Estrutura do projeto

```
my-cloud-messaging/
├── src/
│   ├── main.rs                   # Startup, router, task de limpeza
│   ├── config.rs                 # Configuração via variáveis de ambiente
│   ├── error.rs                  # Tipos de erro com IntoResponse
│   ├── state.rs                  # Estado compartilhado entre handlers
│   ├── db.rs                     # Pool de conexões PostgreSQL
│   ├── models/                   # Structs de request/response
│   ├── handlers/                 # Handlers HTTP (controllers)
│   ├── services/                 # Lógica de negócio (VAPID, envio)
│   ├── repositories/             # Queries no banco de dados
│   └── middleware/               # Autenticação Bearer
├── migrations/                   # Migrations SQL (executadas no startup)
├── Dockerfile                    # Build multi-stage (rust:bookworm → debian:bookworm-slim)
├── docker-compose.yml
└── .env.example
```

## Tecnologias

- **[Axum](https://github.com/tokio-rs/axum)** — framework web async
- **[SQLx](https://github.com/launchbadge/sqlx)** — acesso ao PostgreSQL
- **[web-push](https://github.com/pimeys/rust-web-push)** — protocolo Web Push com VAPID e criptografia AES-128-GCM
- **PostgreSQL 16** — banco de dados
