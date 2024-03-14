package fr.univ_lyon1.info.m1.eliza.model;

import fr.univ_lyon1.info.m1.eliza.model.messageProcessor.MessageProcessor;
import javafx.collections.ObservableList;

/**
 * Store the state of the conversation between eliza and the user.
 * To achieve this, it uses a static observable list called dialogMessages.
 * Each time the list gets updated the list listener in the view is notified
 * and its dialog window is refreshed.
 */
public class DialogState {
    private String userName;
    private ObservableList<MessageData> dialogMessages;
    private final MessageProcessor processor = new MessageProcessor(this);

    /**
     * Initialize the list so that it can be shared with the message searching class
     * of the model without making the list static.
     * @param dialogMessages The dialog list containing the conversation between the user
     *                       and Eliza.
     */
    public void initializeDialogMessagesList(final ObservableList<MessageData> dialogMessages) {
        this.dialogMessages = dialogMessages;
    }

    public String getUserName() {
        return userName;
    }

    public void setUserName(final String userName) {
        this.userName = userName;
    }

    public ObservableList<MessageData> getDialogMessages() {
        return dialogMessages;
    }

    public int getMessageCount() {
        return dialogMessages.size();
    }

    /**
     * Add a new message sent by the user and the answer from eliza to the dialog list.
     * Triggering an update of the view.
     * @param msg The message string.
     * @param msgNumber The message number in the dialog list.
     */
    public void addMessage(final String msg, final int msgNumber) {
        dialogMessages.add(new MessageData(true, msg, msgNumber));

        // Generate answer from eliza
        final String answer = processor.processMessage(msg);
        dialogMessages.add(new MessageData(false, answer, msgNumber + 1));
    }

    /**
     * Remove a message at a certain index. Triggering an update of the view.
     * @param index The message index in the dialog list.
     */
    public void removeMessage(final int index) {
        for (int i = 0; i < dialogMessages.size(); i++) {
            MessageData d = dialogMessages.get(i);
            if (d.getMsgNumber() == index) {
                dialogMessages.remove(d);
                for (int y = i; y < getMessageCount(); y++) {
                    dialogMessages.get(y).decrementMsgNumber();
                }
            }
        }
    }

    /**
     * Remove every message from the dialog list.
     */
    public void clearMessageList() {
        dialogMessages.clear();
    }

    /**
     * When the GUI loads and the conversation starts, eliza says hi.
     */
    public void sayHi() {
        dialogMessages.add(new MessageData(false, "Bonjour", 0));
    }
}
