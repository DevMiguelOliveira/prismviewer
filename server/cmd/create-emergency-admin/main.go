package main

import (
	"crypto/rand"
	"encoding/base64"
	"fmt"
	"log"
	"os"
	"time"

	"golang.org/x/crypto/argon2"

	"prysmview/server/internal/db"
)

const (
	DBPath          = "prysmview.db"
	EmergencyUserID = "emergency@admin.local"
	EmergencyUserName = "Emergency Admin"
)

var DefaultArgon2idParams = struct {
	Memory      uint32
	Iterations  uint32
	Parallelism uint8
	SaltLength  uint32
	KeyLength   uint32
}{
	Memory:      64 * 1024,
	Iterations:  3,
	Parallelism: 2,
	SaltLength:  16,
	KeyLength:   32,
}

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Uso: go run main.go <senha>")
		fmt.Println("Exemplo: go run main.go 'MinhaSenhaSegura!123'")
		os.Exit(1)
	}

	password := os.Args[1]
	if len(password) < 8 {
		log.Fatal("Senha deve ter pelo menos 8 caracteres")
	}

	// Gerar hash Argon2id
	hash, err := hashPasswordArgon2id(password)
	if err != nil {
		log.Fatalf("Erro ao gerar hash: %v", err)
	}

	fmt.Printf("Hash gerado: %s\n", hash)

	// Abrir/criar banco de dados
	database, err := db.Open(DBPath)
	if err != nil {
		log.Fatalf("Erro ao abrir banco: %v", err)
	}
	defer database.Close()

	// Remover operador anterior se existir (limpeza para recriação)
	_ = database.DeleteOperator(EmergencyUserID)

	// Criar operador de emergência
	op := db.Operator{
		ID:           EmergencyUserID,
		Name:         EmergencyUserName,
		PasswordHash: hash,
		CreatedAt:    time.Now(),
	}

	if err := database.CreateOperator(op); err != nil {
		log.Fatalf("Erro ao criar operador: %v", err)
	}

	fmt.Printf("✓ Operador de emergência criado com sucesso!\n")
	fmt.Printf("  ID: %s\n", EmergencyUserID)
	fmt.Printf("  Nome: %s\n", EmergencyUserName)
	fmt.Printf("  Use esta senha para login de emergência no PrysmAdmin\n")
}

func hashPasswordArgon2id(password string) (string, error) {
	params := DefaultArgon2idParams
	salt := make([]byte, params.SaltLength)
	if _, err := rand.Read(salt); err != nil {
		return "", fmt.Errorf("falha ao gerar salt: %w", err)
	}

	hash := argon2.IDKey([]byte(password), salt, params.Iterations, params.Memory, params.Parallelism, params.KeyLength)

	b64Salt := base64.RawStdEncoding.EncodeToString(salt)
	b64Hash := base64.RawStdEncoding.EncodeToString(hash)

	encoded := fmt.Sprintf("$argon2id$v=%d$m=%d,t=%d,p=%d$%s$%s",
		argon2.Version, params.Memory, params.Iterations, params.Parallelism, b64Salt, b64Hash)

	return encoded, nil
}