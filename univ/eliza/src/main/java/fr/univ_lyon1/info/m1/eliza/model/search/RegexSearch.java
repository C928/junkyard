package fr.univ_lyon1.info.m1.eliza.model.search;

import fr.univ_lyon1.info.m1.eliza.model.MessageData;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.collections.ObservableList;

import java.util.ArrayList;
import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * Search messages matching a regex defined in the search query.
 */
public class RegexSearch extends SearchMessage {
    /**
     * Initialize the different classe attribute so that it can be shared between
     * the different model classes.
     * @param dialogMessages The dialog message list.
     * @param searchMessageBackup The search message list.
     * @param isSearchActive The activation status of the search mode.
     */
    public RegexSearch(final ObservableList<MessageData> dialogMessages,
                       final ObservableList<MessageData> searchMessageBackup,
                       final SimpleBooleanProperty isSearchActive) {
        setupClassVariables(dialogMessages, searchMessageBackup, isSearchActive);
    }

    @Override
    public String toString() {
        return "Regex";
    }

    @Override
    public void searchMessage(final String regex) {
        setupSearch();
        List<MessageData> found = new ArrayList<>();
        Pattern pattern = Pattern.compile(".*" + regex + ".*");
        Matcher matcher;
        int index = 0;
        for (MessageData d : getSearchMessageBackup()) {
            matcher = pattern.matcher(d.getMessage());
            if (matcher.matches()) {
                d.setMsgNumber(index);
                found.add(d);
                index++;
            }
        }

        getDialogMessages().addAll(found);
    }
}
