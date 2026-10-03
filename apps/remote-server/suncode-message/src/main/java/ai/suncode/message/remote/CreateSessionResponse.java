package ai.suncode.message.remote;

import ai.suncode.message.EncryptedPayload;
import lombok.Data;
import lombok.EqualsAndHashCode;

@EqualsAndHashCode(callSuper = true)
@Data
public class CreateSessionResponse extends EncryptedPayload {
    private String sessionId;
}
