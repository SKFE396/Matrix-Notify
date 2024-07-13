# M-Notify

A command line tool for sending a message to a matrix room.

## Usage

```bash
./mnotify {config file} {matrix session cache file} {message}
```

## Configuration
An example `config.yaml`:

```yaml
server_name: example.server.com
user_id: '@example_user:example.server.com'
password: 'my-password'
room_name: 'notification room'
```
