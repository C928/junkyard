package fr.univ_lyon1.info.m1.eliza.controller;

import fr.univ_lyon1.info.m1.eliza.model.DialogState;
import fr.univ_lyon1.info.m1.eliza.model.search.RegexSearch;
import fr.univ_lyon1.info.m1.eliza.model.search.SearchMessage;
import fr.univ_lyon1.info.m1.eliza.model.search.SubstringSearch;
import fr.univ_lyon1.info.m1.eliza.model.search.WordSearch;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.concurrent.atomic.AtomicBoolean;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

/**
 * Tests of the controller methods used by the view to communicate with the model
 * and obtain list observers of model data.
 */
public class DialogControllerTest {
    @Test
    void testControllerMethods() {
        // Given
        DialogState dialogState = new DialogState();
        DialogController dialogStateController = new DialogController(dialogState);
        final String firstMessage = "abcd 1234 that's a message";

        // Then
        dialogStateController.sayHi();
        assertThat(dialogStateController.getDialogMessages().get(0).getMessage(), is("Bonjour"));

        dialogStateController.addMessage(firstMessage, 1);
        assertThat(dialogStateController.getDialogMessages().get(1).getMessage(), is(firstMessage));

        dialogStateController.removeMessage(1);
        // Two messages are left, the "Hi" message and the answer from eliza of the first
        // message sent by the user.
        assertThat(dialogStateController.getMessageCount(), is(2));

        dialogStateController.clearMessageList();
        assertThat(dialogStateController.getMessageCount(), is(0));

        ArrayList<SearchMessage> searchTypes = dialogStateController.getSearchTypes();
        assertThat(searchTypes.get(0) instanceof SubstringSearch, is(true));
        assertThat(searchTypes.get(1) instanceof WordSearch, is(true));
        assertThat(searchTypes.get(2) instanceof RegexSearch, is(true));

        SearchMessage substringSearch = searchTypes.get(0);
        startSearchActiveObservable(dialogStateController);
        dialogStateController.searchMessage(substringSearch, "bip boop bip boop");
        dialogStateController.undoSearchMessage();

    }

    /**
     * Test the observable variable "isSearchActive" used to update the search part of the view.
     * @param dialogController The dialog state controller from which we get the observer.
     */
    void startSearchActiveObservable(final DialogController dialogController) {
        AtomicBoolean isFirstSearchActivation = new AtomicBoolean(true);
        dialogController.getSearchActiveObservable()
                .addListener((obs, oldValue, searchIsActive) -> {
            if (isFirstSearchActivation.get()) {
                assertThat(searchIsActive, is(true));
                isFirstSearchActivation.set(false);
            } else {
                assertThat(searchIsActive, is(false));
            }
        });
    }
}
