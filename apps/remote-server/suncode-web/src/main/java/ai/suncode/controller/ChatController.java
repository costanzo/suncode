package ai.suncode.controller;

import ai.suncode.message.ApiBaseRet;
import lombok.extern.slf4j.Slf4j;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

@RestController
@RequestMapping("/chat")
@Slf4j
public class ChatController {

    @PostMapping("/complete")
    public ApiBaseRet<?> chatComplete(@RequestBody String request) {
        return ApiBaseRet.success();
    }
}
