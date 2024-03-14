package fr.univ_lyon1.info.m1.eliza.model.search;

import fr.univ_lyon1.info.m1.eliza.model.MessageData;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.collections.ObservableList;

import java.util.ArrayList;
import java.util.List;

/**
 * Search messages containing a specific word defined in the search query.
 */
public class WordSearch extends SearchMessage {
    /**
     * Initialize the different classe attribute so that it can be shared between
     * the different model classes.
     * @param dialogMessages The dialog message list.
     * @param searchMessageBackup The search message list.
     * @param isSearchActive The activation status of the search mode.
     */
    public WordSearch(final ObservableList<MessageData> dialogMessages,
                      final ObservableList<MessageData> searchMessageBackup,
                      final SimpleBooleanProperty isSearchActive) {
        setupClassVariables(dialogMessages, searchMessageBackup, isSearchActive);
    }

    @Override
    public String toString() {
        return "Word";
    }

    @Override
    public void searchMessage(final String searchedWord) {
        setupSearch();
        List<MessageData> found = new ArrayList<>();
        int index = 0;
        for (MessageData d : getSearchMessageBackup()) {
            String[] words = d.getMessage().split("\\s+");
            for (String word : words) {
                if (word.equals(searchedWord)) {
                    d.setMsgNumber(index);
                    found.add(d);
                    index++;
                    break;
                }
            }
        }

        getDialogMessages().addAll(found);
    }
}
