#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Error codes for FFI functions
 */
typedef enum BridgeErrorCode {
  Success = 0,
  NullPointer = 1,
  InvalidUtf8 = 2,
  SerializationError = 3,
  DeserializationError = 4,
  BlockchainError = 5,
  ConsensusError = 6,
  TransactionError = 7,
  StateError = 8,
  InvalidParameter = 9,
  InternalError = 10,
} BridgeErrorCode;

/**
 * FFI-safe error result
 */
typedef struct BridgeError {
  enum BridgeErrorCode code;
  char *message;
} BridgeError;

/**
 * FFI-safe transaction structure
 */
typedef struct CTransaction {
  const char *from;
  const char *to;
  uint64_t amount;
  uint64_t gas_fee;
  uint64_t timestamp;
  uint64_t nonce;
  const char *signature;
} CTransaction;

/**
 * Serialized data buffer for complex types
 */
typedef struct CByteArray {
  const uint8_t *data;
  uintptr_t len;
} CByteArray;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Initialize the Sultan bridge (call once at startup)
 */
struct BridgeError sultan_bridge_init(void);

/**
 * Shutdown the bridge (cleanup resources)
 */
struct BridgeError sultan_bridge_shutdown(void);

/**
 * Create new blockchain instance
 * Returns: handle ID (> 0) on success, 0 on error
 */
uintptr_t sultan_blockchain_new(struct BridgeError *error);

/**
 * Destroy blockchain instance
 */
struct BridgeError sultan_blockchain_destroy(uintptr_t handle);

/**
 * Get blockchain height
 */
uint64_t sultan_blockchain_height(uintptr_t handle, struct BridgeError *error);

/**
 * Get latest block hash
 */
char *sultan_blockchain_latest_hash(uintptr_t handle, struct BridgeError *error);

/**
 * Free string memory
 */
void sultan_bridge_free_string(char *s);

/**
 * Add transaction to blockchain
 */
bool sultan_blockchain_add_transaction(uintptr_t handle,
                                       struct CTransaction tx,
                                       struct BridgeError *error);

/**
 * Get account balance
 */
uint64_t sultan_blockchain_get_balance(uintptr_t handle,
                                       const char *address,
                                       struct BridgeError *error);

/**
 * Initialize account
 */
bool sultan_blockchain_init_account(uintptr_t handle,
                                    const char *address,
                                    uint64_t balance,
                                    struct BridgeError *error);

/**
 * Create new block
 */
bool sultan_blockchain_create_block(uintptr_t handle,
                                    const char *validator,
                                    struct BridgeError *error);

/**
 * Create new consensus engine
 */
uintptr_t sultan_consensus_new(struct BridgeError *error);

/**
 * Add validator
 */
bool sultan_consensus_add_validator(uintptr_t handle,
                                    const char *address,
                                    uint64_t stake,
                                    struct BridgeError *error);

/**
 * Select next proposer
 */
char *sultan_consensus_select_proposer(uintptr_t handle, struct BridgeError *error);

/**
 * Free byte array memory
 */
void sultan_bridge_free_bytes(struct CByteArray bytes);

/**
 * Free error message memory (must be called from Go side)
 */
void sultan_bridge_free_error(struct BridgeError error);

/**
 * Process ABCI request
 */
struct CByteArray sultan_abci_process(uintptr_t blockchain_handle,
                                      struct CByteArray request_bytes,
                                      struct BridgeError *error);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
