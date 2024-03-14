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
 * Tests for word searching class (WordSearch).
 */
public class WordSearchTest {
    @Test
    void testWordSearchMethods() {
        // Given
        DialogState dialogState = new DialogState();
        ObservableList<MessageData> dialogList = FXCollections.observableArrayList();
        dialogState.initializeDialogMessagesList(dialogList);

        ObservableList<MessageData> searchMessageBackup = FXCollections.observableArrayList();
        SimpleBooleanProperty isSearchActive = new SimpleBooleanProperty(false);

        WordSearch wordSearch = new WordSearch(dialogList, searchMessageBackup, isSearchActive);
        final String firstMessage = "This is the first message 1234";

        // Then
        assertThat(wordSearch.toString(), is("Word"));
        dialogState.addMessage(firstMessage, 0);
        dialogState.addMessage("That's the second message", 1);

        wordSearch.searchMessage("first");
        assertThat(dialogState.getMessageCount(), is(1));
        assertThat(dialogState.getDialogMessages().get(0).getMessage(), is(firstMessage));

        wordSearch.searchMessage("1234");
        assertThat(dialogState.getMessageCount(), is(1));
        assertThat(dialogState.getDialogMessages().get(0).getMessage(), is(firstMessage));

        // This message won't match the first message with word searching
        wordSearch.searchMessage("This is");
        assertThat(dialogState.getMessageCount(), is(0));
        dialogState.clearMessageList();
    }
}
