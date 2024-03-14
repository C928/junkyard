package fr.univ_lyon1.info.m1.eliza.model.search;

import fr.univ_lyon1.info.m1.eliza.model.DialogState;

import fr.univ_lyon1.info.m1.eliza.model.MessageData;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

/**
 * Tests for regex searching class (RegexSearch).
 */
public class RegexSearchTest {
    @Test
    void testRegexSearchMethods() {
        // Given
        DialogState dialogState = new DialogState();
        ObservableList<MessageData> dialogList = FXCollections.observableArrayList();
        dialogState.initializeDialogMessagesList(dialogList);

        ObservableList<MessageData> searchMessageBackup = FXCollections.observableArrayList();
        SimpleBooleanProperty isSearchActive = new SimpleBooleanProperty(false);

        RegexSearch regexSearch = new RegexSearch(dialogList, searchMessageBackup, isSearchActive);
        final String firstMessage = "Some message 1234";
        final String secondMessage = "Hi bob, here is my phone number: +33 1 01 01 01 01";

        // Then
        assertThat(regexSearch.toString(), is("Regex"));
        dialogState.addMessage(firstMessage, 0);

        regexSearch.searchMessage("m.*a.*e 1234");
        assertThat(dialogState.getMessageCount(), is(1));
        assertThat(dialogState.getDialogMessages().get(0).getMessage(), is(firstMessage));

        dialogState.addMessage(secondMessage, 1);
        regexSearch.searchMessage("\\+33 \\d \\d{2} \\d{2} \\d{2} \\d{2}");
        assertThat(dialogState.getMessageCount(), is(1));
        assertThat(dialogState.getDialogMessages().get(0).getMessage(), is(secondMessage));

        regexSearch.searchMessage("This regex search wont find any message");
        assertThat(dialogState.getMessageCount(), is(0));
        dialogState.clearMessageList();
    }
}
