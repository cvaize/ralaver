# Rust admin panel [In process]

## Среда разработки

Среда разработки состоит из:
1) rust:1.82.0
2) nodejs:22.13.1
3) diesel_cli - is rust package

### Команды бекенда
Перед запуском проекта создайте .env файл с переменными окружения из файла .env.example:
```shell
cp .env.example .env
```

Команда запуска среды разработки:
```shell
docker compose -f dev.docker-compose.yaml up -d
```

Команда остановки среды разработки:
```shell
docker compose -f dev.docker-compose.yaml down
```

Команда входа в среду разработки:
```shell
docker compose -f dev.docker-compose.yaml exec dev_tools bash
```
Эта команда будет полезна, например: для применения миграций.

Команда перезапуска среды разработки вместе со сборкой исходников:
```shell
docker compose -f dev.docker-compose.yaml down && docker compose -f dev.docker-compose.yaml up --build -d
```

Команда запуска приложения в среде разработки:
```shell
docker compose -f dev.docker-compose.yaml exec dev_tools cargo run
```

#### Миграции базы данных
Команда запуска миграций:
```shell
diesel migration run
```
```shell
docker compose -f dev.docker-compose.yaml exec dev_tools diesel migration run
```

Команда отката миграций:
```shell
diesel migration revert
```
```shell
docker compose -f dev.docker-compose.yaml exec dev_tools diesel migration revert
```

Команда создания миграций:
```shell
diesel migration generate create_users
```
```shell
docker compose -f dev.docker-compose.yaml exec dev_tools diesel migration generate create_users
```




### Команды фронтенда
Команда для установки зависимостей фронтенда:
```shell
docker compose -f dev.docker-compose.yaml exec dev_tools npm i
```

Команда для сборки фронтенда (перед сборкой не забудьте установить зависимости):
```shell
docker compose -f dev.docker-compose.yaml exec dev_tools npm run build
```

Команда для перезаписи root владельца файлов:
```shell
sudo chown -R $UID:$UID .
```

## Заметки:

Если возникает ошибка diesel: 
`No function or associated item "as_select" found in the current scope for struct "User"` и подобные, 
то нужно подключить методы `use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};`. Например:
```rust
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};

fn run(){
    let mut connection = db_pool.get()
        .map_err(|_| error::ErrorInternalServerError("Db error"))?;

    let results: Vec<User> = crate::schema::users::dsl::users
        .select(User::as_select())
        .limit(1)
        .load::<User>(&mut connection)
        .map_err(|_| error::ErrorInternalServerError("Users load failed."))?;

    let result: Option<&User> = results.get(0);
}
```