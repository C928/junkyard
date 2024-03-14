package fr.univ_lyon1.info.m1.eliza.controller;

import fr.univ_lyon1.info.m1.eliza.model.DialogState;
import fr.univ_lyon1.info.m1.eliza.model.MessageData;
import fr.univ_lyon1.info.m1.eliza.model.search.RegexSearch;
import fr.univ_lyon1.info.m1.eliza.model.search.SearchMessage;
import fr.univ_lyon1.info.m1.eliza.model.search.SubstringSearch;
import fr.univ_lyon1.info.m1.eliza.model.search.WordSearch;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

/**
 * This controller allows to view to communicate with the model and make
 * changes on the dialog state (adding/removing/searching messages).
 * It also contains methods to provide lists observers to the view so that
 * it can be updated on changes of the model state.
 */
public class DialogController {
    private final DialogState dialogState;
    private final ArrayList<SearchMessage> searchTypes = new ArrayList<>();

    /**
     * Constructor for the dialog controller. It initializes the model classes (search types,
     * dialogState) so that the view can interact with the model.
     * @param dialogState The dialog state model class.
     */
    public DialogController(final DialogState dialogState) {
        ObservableList<MessageData> dialogList = FXCollections.observableArrayList();
        dialogState.initializeDialogMessagesList(dialogList);
        this.dialogState = dialogState;

        List<SearchMessage> types = getSearchMessagesTypes(dialogList);
        searchTypes.addAll(types);
    }

    /**
     * Initialize the different search classes with their shared attributes.
     * @param dialogList The shared dialog list.
     * @return The list of search classes.
     */
    private List<SearchMessage> getSearchMessagesTypes(
            final ObservableList<MessageData> dialogList) {
        ObservableList<MessageData> searchMessageBackup = FXCollections.observableArrayList();
        SimpleBooleanProperty isSearchActive = new SimpleBooleanProperty(false);

        return new ArrayList<>(Arrays.asList(
            new SubstringSearch(dialogList, searchMessageBackup, isSearchActive),
            new WordSearch(dialogList, searchMessageBackup, isSearchActive),
            new RegexSearch(dialogList, searchMessageBackup, isSearchActive)
        ));
    }

    /**
     * Called on GUI startup to make eliza say Hi.
     */
    public void sayHi() {
        dialogState.sayHi();
    }

    /**
     * Used by the view to populate its search combo box.
     * @return A list containing different search classes.
     */
    public ArrayList<SearchMessage> getSearchTypes() {
        return searchTypes;
    }

    public ObservableList<MessageData> getDialogMessages() {
        return dialogState.getDialogMessages();
    }

    public SimpleBooleanProperty getSearchActiveObservable() {
        // All search types share the same isSearchActive variable.
        // Because of that, we can get the observer from any of the search type classes.
        return searchTypes.get(0).getSearchActiveObservable();
    }

    public int getMessageCount() {
        return dialogState.getMessageCount();
    }

    /**
     * Method called from the view to remove a remove from the dialog list.
     * @param index The index of the message to be removed
     */
    public void removeMessage(final int index) {
        dialogState.removeMessage(index);
    }

    /**
     * Method called from the view to remove every message from the dialog list.
     * Triggering an update on the view.
     */
    public void clearMessageList() {
        dialogState.clearMessageList();
    }

    /**
     * Method called from the view to add a new user message in the dialog list.
     * Triggering an update on the view.
     * @param msg The content of the message sent by the user.
     * @param msgNumber The message index in the conversation.
     */
    public void addMessage(final String msg, final int msgNumber) {
        dialogState.addMessage(msg, msgNumber);
        // All search types share the same isSearchActive variable.
        // Because of that, we can call setSearchNotActive from any of the search type classes.
        searchTypes.get(0).setSearchNotActive();
    }

    /**
     * Method called from the view to search a message in the dialog list.
     * Triggering an update on the view.
     */
    public void searchMessage(final SearchMessage searchType, final String searchText) {
        searchType.searchMessage(searchText);
        searchType.setSearchActive();
    }

    /**
     * Method called from the view to inform the model that the search query was undone.
     * Triggering an update on the view.
     */
    public void undoSearchMessage() {
        // All search types share the same isSearchActive variable.
        // Because of that, we can call setSearchNotActive from any of the search type classes.
        SearchMessage s = searchTypes.get(0);
        s.undoSearchMessage();
        s.setSearchNotActive();
    }
}
