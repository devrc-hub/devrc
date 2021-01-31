Tasks automation tool for developers
====================================


devrc tasks file based on yaml markup language



## List of tasks

Show list of available tasks
```bash

devrc --tasks
```

Show global variables

```bash

devrc --vars
```


Examples
========


### Simple example

```yaml

task_name: echo "It's work"
```

```bash

devrc task_name
```


### Task with params

```yaml

hello NAME: echo "Hello, {{ NAME }}"

```

```bash

devrc task_name
```


### Task with subcommands

```yaml

run: pass
```


```bash
devrc run stage
```


```bash
devrc prod run

```

```bash
devrc stage run
```
