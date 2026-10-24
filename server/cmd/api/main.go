package main

import (
	"context"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"prysmview/server/internal/api"
	"prysmview/server/internal/auth"
	"prysmview/server/internal/db"
)

func main() {
	// Logging em formato JSON para operacao em producao sem dados sensiveis
	logger := slog.New(slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{
		Level: slog.LevelInfo,
	}))
	slog.SetDefault(logger)

	dbPath := os.Getenv("PRYSMVIEW_DB_PATH")
	if dbPath == "" {
		dbPath = "prysmview.db"
	}

	database, err := db.Open(dbPath)
	if err != nil {
		slog.Error("Falha fatal ao inicializar banco de dados SQLite", "erro", err.Error())
		os.Exit(1)
	}
	defer database.Close()

	tracker := auth.NewNonceTracker()
	limiter := auth.NewIPRateLimiter()

	router := api.NewRouter(database, tracker, limiter)

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	server := &http.Server{
		Addr:         ":" + port,
		Handler:      router,
		ReadTimeout:  15 * time.Second,
		WriteTimeout: 15 * time.Second,
		IdleTimeout:  60 * time.Second,
	}

	// Graceful shutdown
	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt, syscall.SIGTERM)

	go func() {
		slog.Info("Servidor de sinalizacao PrysmView iniciado", "porta", port, "banco", dbPath)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			slog.Error("Erro no servidor HTTP", "erro", err.Error())
			os.Exit(1)
		}
	}()

	<-stop
	slog.Info("Recebido sinal de encerramento, desligando servidor de sinalizacao...")

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := server.Shutdown(ctx); err != nil {
		slog.Error("Falha durante o encerramento gracioso", "erro", err.Error())
	} else {
		slog.Info("Servidor de sinalizacao encerrado com sucesso.")
	}
}
