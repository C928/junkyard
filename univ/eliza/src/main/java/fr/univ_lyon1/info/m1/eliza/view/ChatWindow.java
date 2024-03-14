package fr.univ_lyon1.info.m1.eliza.view;

import fr.univ_lyon1.info.m1.eliza.controller.DialogController;
import fr.univ_lyon1.info.m1.eliza.model.MessageData;
import fr.univ_lyon1.info.m1.eliza.model.search.SearchMessage;
import javafx.collections.ListChangeListener;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.ScrollPane;
import javafx.scene.control.TextField;
import javafx.scene.control.ComboBox;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Pane;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import java.net.URL;

/**
 * Main class of the View (GUI) of the application.
 */
public class ChatWindow {
    private final DialogController controller;
    // Used to store message count in the view (every hBox messages).
    private int msgCount = 0;
    private final VBox dialog;
    private TextField msgText;
    private final Label searchActiveLabel = new Label("No active search");
    private final Button undoButton = new Button("Undo search");

    /**
     * Create the main view of the application.
     */
    public ChatWindow(final DialogController dialogStateController,
                      final Stage stage,
                      final int width,
                      final int height) {
        this.controller = dialogStateController;
        stage.setTitle("Eliza GPT");
        stage.setMinWidth(300);
        stage.setMinHeight(300);

        final VBox root = new VBox(10);
        root.getStyleClass().add("root");
        final Pane search = createTopWidget();
        root.getChildren().add(search);

        ScrollPane dialogScroll = new ScrollPane();
        dialog = new VBox(10);
        dialog.getStyleClass().add("text-area");
        dialogScroll.setContent(dialog);

        // Scroll to bottom by default:
        dialogScroll.vvalueProperty().bind(dialog.heightProperty());
        root.getChildren().add(dialogScroll);
        dialogScroll.setFitToWidth(true);

        final Pane input = createInputWidget();
        root.getChildren().add(input);

        // Everything's ready: add it to the scene and display it
        final Scene scene = new Scene(root, width, height);
        URL css = ChatWindow.class.getResource("/styles/chat-window.css");
        if (css == null) {
            System.err.println("Could not load chat-window.css");
            System.exit(1);
        }

        scene.getStylesheets().add(css.toExternalForm());
        stage.setScene(scene);
        msgText.requestFocus();

        startDialogMessageListener();
        startSearchActiveListener();
        stage.show();
    }

    /**
     * Listen for changes on the observable list (DIALOG_MESSAGES) from the model and
     * update the view to match its content.
     */
    private void startDialogMessageListener() {
        controller.getDialogMessages().addListener(
                (ListChangeListener.Change<? extends MessageData> change) -> {
            if (controller.getMessageCount() == 0) {
                msgCount = 0;
                dialog.getChildren().clear();
            }

            while (change.next()) {
                if (change.wasAdded()) {
                    for (MessageData d : change.getAddedSubList()) {
                        displayMessage(d);
                        msgCount++;
                    }
                }

                if (change.wasRemoved()) {
                    for (MessageData d : change.getRemoved()) {
                        if (!dialog.getChildren().isEmpty()) {
                            dialog.getChildren().remove(d.getMsgNumber());
                            msgCount--;
                        }
                    }
                }
            }
        });
    }

    private void startSearchActiveListener() {
        controller.getSearchActiveObservable().addListener((obs, oldValue, searchIsActive) -> {
            searchActiveLabel.setVisible(!searchIsActive);
            searchActiveLabel.setManaged(!searchIsActive);
            undoButton.setVisible(searchIsActive);
        });
    }

    private void displayMessage(final MessageData data) {
        final HBox messageBox = new HBox();
        //messageBox.getStyleClass().add("msg-box");

        final HBox innerMessageBox = new HBox();
        //innerMessageBox.getStyleClass().add("msg-box");
        innerMessageBox.setSpacing(10);
        innerMessageBox.setAlignment(Pos.CENTER);

        final Label label = new Label(data.getMessage());
        final Button deleteMessageButton = new Button("x");

        Pos position;
        if (data.isUser()) {
            position = Pos.BASELINE_RIGHT;
            deleteMessageButton.getStyleClass().add("delete-user-msg-btn");
            innerMessageBox.getStyleClass().add("user-msg");
        } else {
            position = Pos.BASELINE_LEFT;
            deleteMessageButton.getStyleClass().add("delete-eliza-msg-btn");
            innerMessageBox.getStyleClass().add("eliza-msg");
        }

        innerMessageBox.getChildren().add(label);
        innerMessageBox.getChildren().add(deleteMessageButton);
        messageBox.getChildren().add(innerMessageBox);

        messageBox.setAlignment(position);
        dialog.getChildren().add(messageBox);

        deleteMessageButton.setOnMouseClicked(e -> {
            controller.removeMessage(dialog.getChildren().indexOf(messageBox));
        });
    }

    /**
     * Create the search widgets and the clear messages button.
     */
    private Pane createTopWidget() {
        final HBox firstLine = new HBox();
        final HBox secondLine = new HBox();
        firstLine.setAlignment(Pos.BASELINE_LEFT);
        secondLine.setAlignment(Pos.BASELINE_LEFT);

        TextField searchText = new TextField();
        ComboBox<SearchMessage> searchComboBox = new ComboBox<>();
        searchComboBox.getItems().addAll(controller.getSearchTypes());

        searchText.prefWidthProperty()
                .bind(firstLine.widthProperty()
                        .subtract(searchComboBox.widthProperty()));
        searchComboBox.prefHeightProperty().bind(searchText.heightProperty());

        searchComboBox.setValue(searchComboBox.getItems().get(0));
        firstLine.getChildren().addAll(searchText, searchComboBox);

        undoButton.setVisible(false);
        undoButton.setOnAction(e -> {
            controller.undoSearchMessage();
        });

        final Button searchButton = new Button("Search");
        searchButton.setOnAction(e -> {
            controller.searchMessage(searchComboBox.getValue(), searchText.getText());
        });

        searchText.setOnAction(e -> {
            controller.searchMessage(searchComboBox.getValue(), searchText.getText());
        });

        secondLine.getChildren().addAll(searchButton, searchActiveLabel, undoButton);
        final VBox input = new VBox();
        input.getChildren().addAll(firstLine, secondLine);

        return input;
    }

    private Pane createInputWidget() {
        final Pane inputPane = new HBox();
        msgText = new TextField();
        msgText.setPrefColumnCount(25);
        msgText.setOnAction(e -> {
            controller.addMessage(msgText.getText(), msgCount);
            msgText.setText("");
        });

        final Button clearButton = new Button("Clear");
        clearButton.setOnAction(e -> {
            controller.clearMessageList();
        });

        final Button sendButton = new Button("Send");
        sendButton.setOnAction(e -> {
            controller.addMessage(msgText.getText(), msgCount);
            msgText.setText("");
        });

        msgText.prefWidthProperty()
                .bind(inputPane.widthProperty()
                        .subtract(sendButton.widthProperty())
                        .subtract(clearButton.widthProperty()));
        inputPane.getChildren().addAll(clearButton, msgText, sendButton);
        return inputPane;
    }
}
