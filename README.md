# winwait

Specify a list of process names and winwait will run a command once any process matches one of the names in the list. Once the process closes and there are no longer any processes running that match a process name in the list, winwait will execute another command.

## Usage
```shell
winwait location-of-config.conf
```

See example.conf for an example configuration file.