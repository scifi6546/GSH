```
 *----->X+
 |
 |
 |
\/
Y+
```
# General Protocol
The terminal has two modes append and free write. In append the zero is placed at the end of the last draw command. In free write zero does not move and 
content can write over its self
# Commands
## Set Mode
Sets free write mode:
```
SET FREE
```
Sets append mode
```
SET APPEND
```
## Draw Commands
Draws text at (0,0)
```
"
text
"
```
Draws text at (10,10)
```
DRAW TEXT(10,10)
"

"
```

Draws Line from (10,10) to (20,20)
```
DRAW LINE(10,10,20,20)
```