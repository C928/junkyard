package fr.univ_lyon1.info.m1.eliza.model.search;

import fr.univ_lyon1.info.m1.eliza.model.MessageData;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.collections.ObservableList;

import java.util.ArrayList;
import java.util.List;

/**
 * Search messages containing a word defined in the search query.
 */
public class SubstringSearch extends SearchMessage {
    /**
     * Initialize the different classe attribute so that it can be shared between
     * the different model classes.
     * @param dialogMessages The dialog message list.
     * @param searchMessageBackup The search message list.
     * @param isSearchActive The activation status of the search mode.
     */
    public SubstringSearch(final ObservableList<MessageData> dialogMessages,
                           final ObservableList<MessageData> searchMessageBackup,
                           final SimpleBooleanProperty isSearchActive) {
        setupClassVariables(dialogMessages, searchMessageBackup, isSearchActive);
    }

    @Override
    public String toString() {
        return "Substring";
    }

    @Override
    public void searchMessage(final String subString) {
        setupSearch();
        List<MessageData> found = new ArrayList<>();
        int index = 0;
        for (MessageData d : getSearchMessageBackup()) {
            if (d.getMessage().contains(subString)) {
                d.setMsgNumber(index);
                found.add(d);
                index++;
            }
        }

        getDialogMessages().addAll(found);
    }
}
