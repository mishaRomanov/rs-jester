package main

import (
	"log"
	"net/http"
)

func helloHandler(w http.ResponseWriter, r *http.Request) {
	log.Printf(
		"Request %s: %s %s from %s\n",
		r.Header.Get("X-Request-ID"),
		r.Method,
		r.URL.Path,
		r.RemoteAddr,
	)

	w.Write([]byte(`{"message": "Hello, World!"}`))
}

func main() {
	// Simple multiplexer setup
	mux := http.NewServeMux()
	mux.HandleFunc("/", helloHandler)

	server := http.Server{
		Addr:    ":8080",
		Handler: mux,
	}

	log.Println("Starting the server...")

	if err := server.ListenAndServe(); err != nil {
		log.Fatalf("Server failed to start: %v", err)
	}
}
