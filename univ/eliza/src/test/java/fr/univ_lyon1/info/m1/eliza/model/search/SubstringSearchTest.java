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
 * Tests for substring searching class (SubstringSearch).
 */
public class SubstringSearchTest {
    @Test
    void testSubstringSearchMethods() {
        // Given
        DialogState dialogState = new DialogState();
        ObservableList<MessageData> dialogList = FXCollections.observableArrayList();
        dialogState.initializeDialogMessagesList(dialogList);

        ObservableList<MessageData> searchMessageBackup = FXCollections.observableArrayList();
        SimpleBooleanProperty isSearchActive = new SimpleBooleanProperty(false);

        SubstringSearch substringSearch =
                new SubstringSearch(dialogList, searchMessageBackup, isSearchActive);
        final String firstMessage = "Aaaa b1234bb c ddd eeeeee";

        // Then
        assertThat(substringSearch.toString(), is("Substring"));
        dialogState.addMessage(firstMessage, 0);
        dialogState.addMessage("some other message", 1);

        substringSearch.searchMessage("b1234bb");
        assertThat(dialogState.getMessageCount(), is(1));
        assertThat(dialogState.getDialogMessages().get(0).getMessage(), is(firstMessage));

        substringSearch.searchMessage("b c ddd ee");
        assertThat(dialogState.getMessageCount(), is(1));
        assertThat(dialogState.getDialogMessages().get(0).getMessage(), is(firstMessage));

        substringSearch.searchMessage("This substring search wont find any message");
        assertThat(dialogState.getMessageCount(), is(0));
    }
}
