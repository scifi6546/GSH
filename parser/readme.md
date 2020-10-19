# Protocol
The program uses a binary format. The protocol is shown below:
```
|@+0 | @+1 | @+2 | @+3 | @+4 | @+5 | @+6 | @+7|
|----|-----|-----|-----|-----|-----|-----|----|
|<-----Data Type------>|<---Payload Length--->|
|---------------Data--------------------------|
                    .
                    .
                    .
|---------------Data--------------------------|
```
# Datatypes
## Text
Datatype: 0x0

Data Contents: raw utf-8