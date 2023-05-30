# shrine
Password manager written in rust

# Command-line Usage

### Initialize your shrine
```sh
shrine init
```

### Add secrets
```shell
shrine set personal/github mypassword
shrine set personal/email/me@myhost.net mySecurePassword
shrine set personal/email/me@gmail.com mySecurePassword
```

### List secrets
```shell
shrine ls
shrine ls personal/email/*
```

### Delete secrets
```shell
shrine rm personal/email/me@myhost.net
shrine rm personal/email/*
```

## Configure git integration
```shell
shrine config git/url "https://github.com/cpollet/shrine-secrets.git"
shrine config git/username "cpollet"
shrine config git/token "github token"
shrine config git/email "cpollet@users.noreply.github.com"
shrine config git/commit/auto false
shrine config git/commit/quiet true
shrine config git/push/auto false
```
