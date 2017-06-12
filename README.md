# jenkins-build-stats

> a tool for mining information out of jenkins

## configuration

The following env vars are required

| Name              | Description                       |
|-------------------|-----------------------------------|
| JENKINS_HOST      | jenkins host ( including scheme ) |
| JENKINS_USERNAME  | jenkins username                  |
| JENKINS_PASSWORD  | jenkins password                  |
| JOB               | jenkins job name                  |


## Usage

For now, just run as a cargo main

```bash
$ JENKINS_HOST=xxx JENKINS_USERNAME=xxx JENKINS_PASSWORD=xxx JOB=xxx cargo run
```
