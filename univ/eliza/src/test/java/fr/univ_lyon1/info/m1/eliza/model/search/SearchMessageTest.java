package fr.univ_lyon1.info.m1.eliza.model.search;

import fr.univ_lyon1.info.m1.eliza.model.DialogState;
import fr.univ_lyon1.info.m1.eliza.model.MessageData;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import org.junit.jupiter.api.Test;

import java.util.concurrent.atomic.AtomicBoolean;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

/**
 * Tests for the word searching class (SearchMessage).
 */
public class SearchMessageTest {
    @Test
    void testSearchMessageMethods() {
        // Given
        DialogState dialogState = new DialogState();
        ObservableList<MessageData> dialogList = FXCollections.observableArrayList();
        dialogState.initializeDialogMessagesList(dialogList);

        ObservableList<MessageData> searchMessageBackup = FXCollections.observableArrayList();
        SimpleBooleanProperty isSearchActive = new SimpleBooleanProperty(false);
        SubstringSearch substringSearch =
                new SubstringSearch(dialogList, searchMessageBackup, isSearchActive);
        startSearchActiveObservable(substringSearch);
        final String firstMessage = "some message 1234";
        final String secondMessage = "some other message 5678";

        // Then
        dialogState.addMessage(firstMessage, 0);
        dialogState.addMessage(secondMessage, 1);
        substringSearch.searchMessage("some message 1234");

        substringSearch.setSearchActive();
        assertThat(dialogState.getMessageCount(), is(1));

        substringSearch.undoSearchMessage();
        substringSearch.setSearchNotActive();
        // 2 user messages + 2 eliza responses
        assertThat(dialogState.getMessageCount(), is(4));

        assertThat(dialogState.getDialogMessages().get(0).getMessage(), is(firstMessage));
        assertThat(dialogState.getDialogMessages().get(2).getMessage(), is(secondMessage));
        dialogState.clearMessageList();
    }

    /**
     * Test the observable variable "isSearchActive" used to update the search part of the view.
     * @param substringSearch The search message model.
     */
    void startSearchActiveObservable(final SearchMessage substringSearch) {
        AtomicBoolean isFirstSearchActivation = new AtomicBoolean(true);
        substringSearch.getSearchActiveObservable().addListener((obs, oldValue, searchIsActive) -> {
            if (isFirstSearchActivation.get()) {
                assertThat(searchIsActive, is(true));
                isFirstSearchActivation.set(false);
            } else {
                assertThat(searchIsActive, is(false));
            }
        });
    }
}
