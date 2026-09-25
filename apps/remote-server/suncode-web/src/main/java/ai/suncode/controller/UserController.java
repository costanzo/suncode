package ai.suncode.controller;

import ai.suncode.message.ApiBaseRet;
import lombok.extern.slf4j.Slf4j;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

@RestController
@RequestMapping("/user")
@Slf4j
public class UserController {

    @PostMapping("/login")
    public ApiBaseRet<?> wechatLogin(@RequestBody String request) {
        log.info("login request: {}", request);
        return ApiBaseRet.success();
    }
}
