package fr.univ_lyon1.info.m1.eliza.model;

import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

/**
 * Tests for the DialogState class which is in charge of keeping the state
 * of the conversation between the user and eliza.
 */
public class DialogStateTest {
    @Test
    void testDialogStateMethods() {
        DialogState s = new DialogState();
        ObservableList<MessageData> dialogList = FXCollections.observableArrayList();
        s.initializeDialogMessagesList(dialogList);

        assertThat(s.getMessageCount(), is(0));
        s.addMessage("Some message 1234", 0);
        // user message + eliza response
        assertThat(s.getMessageCount(), is(2));

        assert (s.getUserName() == null);
        s.setUserName("bob");
        assertThat(s.getUserName(), is("bob"));

        assertThat(s.getDialogMessages().get(0).getMessage(), is("Some message 1234"));
        s.addMessage("Some other message", 1);
        assertThat(s.getMessageCount(), is(4));

        s.clearMessageList();
        assertThat(s.getMessageCount(), is(0));

        List<String> testMessages =
                new ArrayList<>(Arrays.asList("abcd", "efgh", "ijkl", "mnop", "qrst"));
        for (int i = 0, y = 0; i < 5; i++, y += 2) {
            s.addMessage(testMessages.get(i), y);
        }
        assertThat(s.getMessageCount(), is(10));

        // Remove the first user message
        s.removeMessage(0);
        testMessages.remove(0);

        // Remove every message from eliza
        s.removeMessage(0);
        s.removeMessage(1);
        s.removeMessage(2);
        s.removeMessage(3);
        s.removeMessage(4);

        // Remove the 4th user message
        s.removeMessage(3);
        testMessages.remove(3);

        // We iterate through the user messages and verify that the message number
        // and content are what they should be.
        for (int i = 1; i < 3; i++) {
            MessageData m = s.getDialogMessages().get(i);
            assertThat(m.getMessage(), is(testMessages.get(i)));
            assertThat(m.getMsgNumber(), is(i));
        }

        s.clearMessageList();
        assertThat(s.getMessageCount(), is(0));

        s.sayHi();
        assertThat(s.getDialogMessages().get(0).getMessage(), is("Bonjour"));
        assertThat(s.getDialogMessages().get(0).getMsgNumber(), is(0));

        s.clearMessageList();
        assertThat(s.getMessageCount(), is(0));
    }
}
