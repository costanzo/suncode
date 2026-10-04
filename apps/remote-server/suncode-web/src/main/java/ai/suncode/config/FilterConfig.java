package ai.suncode.config;

import ai.suncode.common.http.ContextFilter;
import ai.suncode.filter.DesktopAuthFilter;
import ai.suncode.filter.MobileAuthFilter;
import ai.suncode.filter.RequestResponseLoggingFilter;
import ai.suncode.service.RemoteAuthService;
import ai.suncode.service.RemoteRelayService;
import org.springframework.boot.web.servlet.FilterRegistrationBean;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class FilterConfig {
    @Bean
    public FilterRegistrationBean<RequestResponseLoggingFilter> requestResponseLoggingFilterFilterFilterRegistration() {
        FilterRegistrationBean<RequestResponseLoggingFilter> registrationBean = new FilterRegistrationBean<>();
        registrationBean.setFilter(new RequestResponseLoggingFilter());
        registrationBean.addUrlPatterns("/*");
        registrationBean.setName("requestResponseLoggingFilter");
        registrationBean.setOrder(2);
        return registrationBean;
    }

    @Bean
    public FilterRegistrationBean<ContextFilter> contextFilterRegistration() {
        FilterRegistrationBean<ContextFilter> registrationBean = new FilterRegistrationBean<>();
        registrationBean.setFilter(new ContextFilter());
        registrationBean.addUrlPatterns("/*");
        registrationBean.setName("contextFilter");
        registrationBean.setOrder(1);
        return registrationBean;
    }

    @Bean
    public FilterRegistrationBean<DesktopAuthFilter> desktopAuthFilterRegistration(
            RemoteAuthService authService,
            RemoteRelayService relayService) {
        FilterRegistrationBean<DesktopAuthFilter> registrationBean = new FilterRegistrationBean<>();
        registrationBean.setFilter(new DesktopAuthFilter(authService, relayService));
        registrationBean.addUrlPatterns("/v1/desktop/*");
        registrationBean.setName("desktopAuthFilter");
        registrationBean.setOrder(2);
        return registrationBean;
    }

    @Bean
    public FilterRegistrationBean<MobileAuthFilter> mobileAuthFilterRegistration(
            RemoteAuthService authService,
            RemoteRelayService relayService) {
        FilterRegistrationBean<MobileAuthFilter> registrationBean = new FilterRegistrationBean<>();
        registrationBean.setFilter(new MobileAuthFilter(authService, relayService));
        registrationBean.addUrlPatterns("/v1/mobile/*");
        registrationBean.setName("mobileAuthFilter");
        registrationBean.setOrder(2);
        return registrationBean;
    }
}
