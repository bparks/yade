# yade

`yade` stands for Yet Another Database Engine. The thing is... it's not really
a database engine at all. But `yade` is too good of a name to pass up. What
can I say? I'm yaded.

## What does it do?

The intent is for yade to provide a MySQL-compatible interface on top of YAML
files. This is useful, for instance, if you want to store your CMS config in
a git repository.

**NOTE**: We're not there yet. Currently, we just have a service that a mysql client will successfully connect to and get nothing back from any query.

Run yade like

    yade ./path/to/yaml/files

Where the folder containing the YAML files has a structure something like

```
./path/to/yaml/files
  database_name
    table1
      schema.yaml
      1.yaml
      2.yaml
      ...
    table2
      schema.yaml
      ...
    ...
  ...
```
