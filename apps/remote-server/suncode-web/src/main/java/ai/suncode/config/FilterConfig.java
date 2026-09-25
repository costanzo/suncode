package ai.suncode.config;

import ai.suncode.common.http.ContextFilter;
import org.springframework.boot.web.servlet.FilterRegistrationBean;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class FilterConfig {
    @Bean
    public FilterRegistrationBean<ContextFilter> filterRegistrationBean() {
        FilterRegistrationBean<ContextFilter> registrationBean = new FilterRegistrationBean<>();
        registrationBean.setFilter(new ContextFilter());
        registrationBean.addUrlPatterns("/*");
        registrationBean.setName("contextFilter");
        registrationBean.setOrder(1);
        return registrationBean;
    }
}
