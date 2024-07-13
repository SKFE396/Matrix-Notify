# M-Notify

A simple command line tool for sending a message to a matrix room.

## Usage

```bash
./mnotify {config file} {matrix session cache file} {message}
```

The matrix session is cached in a JSON file so that the program won't need to create a new session every time you run this program.

**IMPORTANT:** you need to make sure that **only you** have the access to the config file and the session cache file, or the others will be able to log in to your account.

## Configuration
An example `config.yaml`:

```yaml
server_name: example.server.com
user_id: '@example_user:example.server.com'
password: 'my-password'
room_name: 'notification room'
```
