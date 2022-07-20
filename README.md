# Java-Flow-Analyzer

Java-Flow-Analyzer analyzes Semantic Flows of Java Programs.

## How Does it Work?

Java-Flow-Analyzer is written in Rust. It uses the tree-sitter C++ binding to 
obtain the AST per java file. The Call and Control Flow edges are calculated on statement-level
for easy querying.
The transitive closure of the Flow Graph is calculated using a logical programming Language written in Rust called *Crepe* .

## How To use?
First build the tool:
```
cargo build
```

Place a TOML Configuration file inside your Java Project.

E.g.

```
project = "src/"

[[nodes]]
name = "dynamicClassLoad"
identifier = "local_variable_declaration"
code = ".*Class.forName.*"

[[nodes]]
name = "someSource"
identifier = "method_declaration"
code = ".*Response.*"

[[flows]]
from = "someSource"
to = "dynamicClassLoad"

```

And call it using:
```
rustparse --path=<path to config>
```

The output may look like following:
```
"someSource" reaches "dynamicClassLoad"
Source src/fixtures/welcoemailsubservice/src/main/java/com/welcohealth/email/service/UserRestDefinition.java 86:112
Target src/fixtures/welcoemailsubservice/src/main/java/com/welcohealth/email/service/EmailDAO.java 95:95
____________________________________
"someSource" reaches "dynamicClassLoad"
Source src/fixtures/welcoemailsubservice/src/main/java/com/welcohealth/email/service/UserRestDefinition.java 86:112
Target src/fixtures/welcoemailsubservice/src/main/java/com/welcohealth/email/service/EmailDAO.java 199:199
____________________________________
"someSource" reaches "dynamicClassLoad"
Source src/fixtures/welcoemailsubservice/src/main/java/com/welcohealth/email/service/UserRestDefinition.java 86:112
Target src/fixtures/welcoemailsubservice/src/main/java/com/welcohealth/email/service/EmailDAO.java 302:302
____________________________________
```

## To run the tests

```
cargo test
```
