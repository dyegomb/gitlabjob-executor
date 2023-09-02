[![Build](https://github.com/dyegomb/gitlabjob-executor/actions/workflows/build.yml/badge.svg)](https://github.com/dyegomb/gitlabjob-executor/actions/workflows/build.yml)

# gitlabjob

It's a workaround until conclusion of <https://gitlab.com/gitlab-org/gitlab/-/issues/17718>,
you can create manual jobs that would be started by this program.

Its proposal is to execute manual jobs inside a Gitlab group or project, so you can queue a
manual job that will be started in a proper time by this program.

### How to use
> minimal docker image: `ghcr.io/dyegomb/gitlabjob-executor:latest`

Basically you have to feed the _.env_[^note] file as example below.

[^note]: You can change file name to read with the environment variable *`ENV_FILE`*.

```ini
private_token="XXXXXXXXXXXXX"
base_url="https://gitlab.com/"
project_id=123
group_id=1
production_tag_key="PROD_TAG" # Variable to look for in a pipeline
max_wait_time=1800 # Max waiting time for a job in seconds

[smtp]
server="mail.com"
user="user"
from="user@mail.com"
to="destination@mail.com"
subject="[Subject Prefix] "
pass="Secret"
```

It also supports definition from environment variables, whom **takes precedence**.

The SMTP section is only needed if you want to receive report emails.
SMTP settings from environment variables must has `SMTP_` prefix.

