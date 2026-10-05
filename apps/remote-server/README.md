# SunCode Remote Server

## Requirements

- JDK 21
- Maven 3.9 or later

## Start locally

From this directory, run:

```bash
export JAVA_HOME=$(/usr/libexec/java_home -v 21)
export PATH="$JAVA_HOME/bin:$PATH"

mvn -pl suncode-web -am install -DskipTests

cd suncode-web
mvn spring-boot:run \
  -Dspring-boot.run.main-class=ai.suncode.SunCodeWebApplication \
  -Dspring-boot.run.jvmArguments="-Xmx512m"
```

The server listens on `http://localhost:8080/remote-server` by default.

`-Xmx512m` limits the application JVM's Java heap to 512 MB. It does not limit Maven's own JVM or non-heap/native memory used by the application process.

