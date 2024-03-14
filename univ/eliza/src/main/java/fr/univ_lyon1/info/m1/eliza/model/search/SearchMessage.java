package fr.univ_lyon1.info.m1.eliza.model.search;

import fr.univ_lyon1.info.m1.eliza.model.MessageData;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.collections.ObservableList;

/**
 * Store the backup search message list used to undo a search.
 * When a searched query is sent to one of its derived classes,
 * every message contained in the dialog list are stored in the backup
 * list to be retrieved later if the search gets undone.
 */
public abstract class SearchMessage {
    private ObservableList<MessageData> searchMessageBackup;
    private ObservableList<MessageData> dialogMessages;
    private SimpleBooleanProperty isSearchActive;

    /**
     * Message searching function signature implemented inside each derived
     * classes with a different searching algorithm.
     * @param searchedText The text searched by the user.
     */
    public abstract void searchMessage(String searchedText);

    /**
     * This method will be called by the controller for the view so that the
     * search combo box will know the name of the searching algorithms.
     * @return a String containing the searching algorithm of the derived class.
     */
    public abstract String toString();

    protected void setupSearch() {
        searchMessageBackup.clear();
        searchMessageBackup.addAll(dialogMessages);
        dialogMessages.clear();
    }

    protected ObservableList<MessageData> getSearchMessageBackup() {
        return searchMessageBackup;
    }

    protected ObservableList<MessageData> getDialogMessages() {
        return dialogMessages;
    }

    protected void setupClassVariables(final ObservableList<MessageData> dialogMessages,
                                       final ObservableList<MessageData> searchMessageBackup,
                                       final SimpleBooleanProperty isSearchActive) {
        this.dialogMessages = dialogMessages;
        this.searchMessageBackup = searchMessageBackup;
        this.isSearchActive = isSearchActive;
    }

    /**
     * Display the messages present before the search query was executed.
     */
    public void undoSearchMessage() {
        dialogMessages.clear();
        dialogMessages.addAll(searchMessageBackup);
    }

    public SimpleBooleanProperty getSearchActiveObservable() {
        return isSearchActive;
    }

    /**
     * Activate search mode. This will trigger an update of the view
     * which will then display an "undo" button to undo the search and
     * restore the previous conversation.
     */
    public void setSearchActive() {
        isSearchActive.set(true);
    }

    /**
     * Deactivate search mode. This will trigger an update of the view
     * which will then display a label containing the text "No active search".
     */
    public void setSearchNotActive() {
        isSearchActive.set(false);
    }
}
