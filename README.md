# gitjob-executor

It is a workaround until conclusion of https://gitlab.com/gitlab-org/gitlab/-/issues/17718, you can create manual jobs that would be started by this python work.

Its proposal is to execute manual jobs inside Gitlab group or project, so you can queue a manual job that will be started in a proper time by this program.

## How to use
Basically you have to feed the _.env_ file as example below.

```ini
PRIVATE_TOKEN="XXXXXXXXXXXXX"
BASE_URL="https://gitlab.company.com/"
PROJECT_ID="123" # or GROUP_ID="1"
```
If you inform _GROUP_ID_, the _PROJECT_ID_ will be ignored and the manuals jobs of the pipeline projects inside this group will be started.

Another if is when this parameter has an already setted environment variable with the same name, the value on the ".env" will be ignored.

Then you can execute ```python3 executor.py```, you also can use the Gitlab Schedule to execute it regurlarly.