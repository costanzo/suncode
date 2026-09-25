package ai.suncode;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;

@SpringBootApplication(scanBasePackages = "ai.suncode")
public class SunCodeWebApplication {
    public static void main(String[] args) {
        SpringApplication.run(SunCodeWebApplication.class, args);
    }
}
