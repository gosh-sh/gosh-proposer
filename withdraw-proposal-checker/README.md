To run deposit-proposal-checker you need to export this env variables:
  `ETH_NETWORK` - ETH endpoint (usually get from .env);
  `ETH_FUNCTION_NAME` - ELock deposit function endpoint (usually get from .env);
  `ROOT_FUNCTION_NAME` - GOSH Token Root burn funtion name;
  `GOSH_ENDPOINTS` - GOSH endpoints;
  `ETH_CONTRACT_ADDRESS` - Elock address in ETH network;
  `ROOT_ADDRESS` - Token root address in GOSH network;
  `ETH_PRIVATE_KEY_PATH` - Path to the file with validators ETH private key;
  `ETH_CONFIRMATIONS_CNT` - number of ETH block confirmations (default value is 1).