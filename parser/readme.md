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

## Figure
Datatype: 0x1
Data contents
```
|@+0 | @+1 | @+2 | @+3 | @+4 | @+5 | @+6 | @+7|
|----|-----|-----|-----|-----|-----|-----|----|
|<---x dim (pixels)--->|<---y dim (pixels)--->|
|---------------Data--------------------------|
                    .
                    .
                    .
|---------------Data--------------------------|
```
The data contains a list of elements in the figure.
### Figure Contents Format
```
|@+0 | @+1 | @+2 | @+3 | @+4 | @+5 | @+6 | @+7|
|----|-----|-----|-----|-----|-----|-----|----|
|<---Element Type----->|<---Payload Length--->|
|<--x start (pixels)-->|<--y start (pixels)-->|
|---------------Data--------------------------|
                    .
                    .
                    .
|---------------Data--------------------------|
```
### Picture Figure Element
Element type: 0
Data is png encoded bytes

### line Figure Element
element type: 1
```
|@+0 | @+1 | @+2 | @+3 | @+4 | @+5 | @+6 | @+7|
|----|-----|-----|-----|-----|-----|-----|----|
|<---- rgba color ---->|<--thickness (f32) -->|
|<-x cordinate (f32)-->|<-y cordinate (f32)-->|
                    .
                    .
                    .
|<-x cordinate (f32)-->|<-y cordinate (f32)-->|
```