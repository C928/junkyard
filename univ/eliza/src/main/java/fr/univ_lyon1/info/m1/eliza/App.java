package fr.univ_lyon1.info.m1.eliza;

import fr.univ_lyon1.info.m1.eliza.controller.DialogController;
import fr.univ_lyon1.info.m1.eliza.model.DialogState;
import fr.univ_lyon1.info.m1.eliza.view.ChatWindow;
import javafx.application.Application;
import javafx.stage.Stage;

/**
 * Main class for the application (structure imposed by JavaFX).
 */
public class App extends Application {
    public static final int HEIGHT_WINDOW = 600;
    public static final int WIDTH_WINDOW = 600;

    /**
     * With javafx, start() is called when the application is launched.
     */
    @Override
    public void start(final Stage stage) {
        DialogState dialogState = new DialogState();
        DialogController dialogStateController = new DialogController(dialogState);

        new ChatWindow(dialogStateController, stage, WIDTH_WINDOW, HEIGHT_WINDOW);
        // Second view (uncomment to activate)
        // new ChatWindow(dialogStateController, new Stage(), 400, 400);

        // sayHi() was removed from the view and placed here so that when 2 views
        // are created eliza says hi only once (without having to pass a sayHi boolean
        // to the views).
        dialogStateController.sayHi();
    }

    /**
     * A main method in case the user launches the application using
     * App as the main class.
     * @param args are the command line arguments.
     */
    public static void main(final String[] args) {
        launch(args);
    }
}
