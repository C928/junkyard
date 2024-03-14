package fr.univ_lyon1.info.m1.eliza.model;

/**
 * Wrapper for messages storing diverse information.
 *  - sender
 *  - message content
 *  - its associated number in the message list
 */
public class MessageData {
    private final boolean isUser;
    private final String msg;
    private int msgNumber;

    /**
     * Create a new message with its associated information.
     */
    public MessageData(final boolean isUser, final String msg, final int msgNumber) {
        this.isUser = isUser;
        this.msg = msg;
        this.msgNumber = msgNumber;
    }

    public boolean isUser() {
        return isUser;
    }

    public String getMessage() {
        return msg;
    }

    public int getMsgNumber() {
        return msgNumber;
    }

    /**
     * Used on every element that follows an element that was removed.
     */
    public void decrementMsgNumber() {
        this.msgNumber--;
    }

    public void setMsgNumber(final int msgNumber) {
        this.msgNumber = msgNumber;
    }
}
