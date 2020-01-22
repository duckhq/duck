# Duck

Duck is a build system agnostic build monitor written in Rust (backend server) and Vue (frontend). It also support sending messages to Slack and controlling [Philips Hue](https://www2.meethue.com/) lights.

![The frontend](res/images/frontend.png)

## Running

The absolute easiest way of getting started with Duck is to use the docker image. You will have to provide a configuration file for it to work.

```
> docker run --rm -it -v ./data:/data -p 8080:15825 spectresystems/duck:latest --config /data/duck.json
```

## How it works

The server part is responsible for collecting information from different `collectors` and forward interesting events to `observers`. The aggregated information is available for other services (such as the Vue frontend) via a HTTP API.

Observers can either be dependent on events from all collectors, or opt in to one or more collectors. This makes it easy to setup team specific build lights or Slack integration that's only dependent on specific collectors.

![Overview](res/images/overview.svg)

## Configuration

Below is an example configuration that specifies multiple collectors and observers. By specifying the schema you will get autocomplete support in editors that support it like Visual Studio Code.

```json
{
    "$schema": "https://raw.githubusercontent.com/spectresystems/duck/master/config/v0.3/schema.json",
    "interval": 30,
    "collectors": [
        {
            "teamcity": {
                "id": "teamcity_local",
                "serverUrl": "http://127.0.0.1:8111/",
                "credentials": "guest",
                "builds": [
                    "My_Project_Definition",
                    "My_Other_Build_Definition"
                ]
            }
        },
        {
            "teamcity": {
                "id": "teamcity_other",
                "enabled": false,
                "serverUrl": "192.168.0.1:8111",
                "credentials": {
                    "basic": {
                        "username": "foo",
                        "password": "bar"
                    }
                },
                "builds": [
                    "My_Third_Build_Definition"
                ]
            }
        },
        {
            "azure": {
                "id": "azure_cake",
                "enabled": false,
                "organization": "cake-build",
                "project": "Cake",
                "credentials": "anonymous",
                "definitions": [ "1", "3", "5" ],
                "branches": [
                    "refs/heads/develop",
                    "refs/heads/main"
                ]
            }
        },
        {
            "azure": {
                "id": "azure_private",
                "enabled": false,
                "organization": "some-organization",
                "project": "some-project",
                "credentials": {
                    "pat": "MY-PERSONAL-ACCESS-TOKEN"
                },
                "definitions": [ "1" ],
                "branches": [
                    "refs/heads/master"
                ]
            }
        },
        {
            "octopus": {
                "id": "octopus_local",
                "serverUrl": "http://localhost:9000/",
                "credentials": {
                    "apiKey": "MY-API-KEY"
                },
                "projects": [
                    {
                        "projectId": "my-project",
                        "environments": [
                            "Development", 
                            "Staging",
                            "Production"
                        ]
                    }
                ]
            }
        }
    ],
    "observers": [
        {
            "hue": {
                "id": "hue_team_1_builds",
                "collectors": [
                    "teamcity_local"
                ],
                "hubUrl": "http://192.168.1.99",
                "username": "THE-HUE-USERNAME",
                "lights": [ "1" ]
            }
        },
        {
            "hue": {
                "id": "hue_all_builds_dimmed",
                "hubUrl": "http://192.168.1.99",
                "username": "THE-HUE-USERNAME",
                "brightness": 50,
                "lights": [ "2", "3" ]
            }
        },
        {
            "slack": {
                "id": "slack_team1",
                "collectors": [ "teamcity_local" ],
                "credentials": {
                    "webhook": {
                        "url": "https://hooks.slack.com/services/MY-WEBHOOK-URL"
                    }
                }
            }
        },
        {
            "mattermost": {
                "id": "mattermost",
                "channel": "build-status",
                "credentials": {
                    "webhook": {
                        "url": "https://mattermost.example.com"
                    }
                }
            }
        }
    ]
}
```