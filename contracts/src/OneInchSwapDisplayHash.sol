// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

import "./Display.sol";

library OneInchSwapDisplayHash {
    function getOneInchSwapDisplayHash(address router) internal pure returns (bytes32) {
        return Display.display(
            address(router),
            "function swap(address executor, (address srcToken, address dstToken, address srcReceiver, address dstReceiver, uint256 amount, uint256 minReturnAmount, uint256 flags) desc, bytes data)",
            "$labels.swap_title",
            "$labels.swap_description",
            abi.encode(
                // Field 1: You Send
                Display.tokenAmountField(
                    "$labels.you_send",
                    "$labels.desc_you_send",
                    hex"",
                    "$locals.desc.srcToken",
                    "$locals.desc.amount"
                ),
                // Field 2: You Receive (Minimum)
                Display.tokenAmountField(
                    "$labels.you_receive_minimum",
                    "$labels.desc_you_receive_minimum",
                    hex"",
                    "$locals.desc.dstToken",
                    "$locals.desc.minReturnAmount"
                ),
                // Field 3: Recipient
                Display.addressField(
                    "$labels.recipient",
                    "$labels.desc_recipient",
                    hex"",
                    "$locals.desc.dstReceiver"
                ),
                // Field 4: Swap Options
                Display.bitmaskField(
                    "$labels.swap_options",
                    "$labels.desc_swap_options",
                    hex"",
                    "$locals.desc.flags",
                    abi.encodePacked(
                        Display.entry("#0", "$labels.partial_fill_enabled")
                    )
                )
            ),
            // Labels
            abi.encode(
                // en
                Display.labels(
                    "en",
                    abi.encode(
                        Display.entry("swap_title", "1inch Swap"),
                        Display.entry("swap_description", "Exchange one token for another using the 1inch aggregation protocol"),
                        Display.entry("you_send", "You Send"),
                        Display.entry("you_receive_minimum", "You Receive (Minimum)"),
                        Display.entry("recipient", "Recipient"),
                        Display.entry("swap_options", "Swap Options"),
                        Display.entry("partial_fill_enabled", "Partial fill allowed"),
                        Display.entry("desc_you_send", "The exact amount you will send from your wallet"),
                        Display.entry("desc_you_receive_minimum", "The minimum amount you will receive. You may receive more if the exchange rate improves"),
                        Display.entry("desc_recipient", "The wallet address that will receive the swapped tokens"),
                        Display.entry("desc_swap_options", "Additional swap execution settings"),
                        Display.entry("desc_partial_fill", "If enabled, the swap can be partially filled if the full amount cannot be executed")
                    )
                ),
                // es
                Display.labels(
                    "es",
                    abi.encode(
                        Display.entry("swap_title", unicode"Intercambio 1inch"),
                        Display.entry("swap_description", unicode"Intercambia un token por otro usando el protocolo de agregación 1inch"),
                        Display.entry("you_send", unicode"Envías"),
                        Display.entry("you_receive_minimum", unicode"Recibes (Mínimo)"),
                        Display.entry("recipient", unicode"Destinatario"),
                        Display.entry("swap_options", unicode"Opciones de Intercambio"),
                        Display.entry("partial_fill_enabled", unicode"Llenado parcial permitido"),
                        Display.entry("desc_you_send", unicode"La cantidad exacta que enviarás desde tu billetera"),
                        Display.entry("desc_you_receive_minimum", unicode"La cantidad mínima que recibirás. Puedes recibir más si mejora el tipo de cambio"),
                        Display.entry("desc_recipient", unicode"La dirección de billetera que recibirá los tokens intercambiados"),
                        Display.entry("desc_swap_options", unicode"Configuraciones adicionales de ejecución del intercambio"),
                        Display.entry("desc_partial_fill", unicode"Si está habilitado, el intercambio puede llenarse parcialmente si no se puede ejecutar la cantidad completa")
                    )
                ),
                // ru
                Display.labels(
                    "ru",
                    abi.encode(
                        Display.entry("swap_title", unicode"Обмен 1inch"),
                        Display.entry("swap_description", unicode"Обменяйте один токен на другой, используя протокол агрегации 1inch"),
                        Display.entry("you_send", unicode"Вы отправляете"),
                        Display.entry("you_receive_minimum", unicode"Вы получаете (Минимум)"),
                        Display.entry("recipient", unicode"Получатель"),
                        Display.entry("swap_options", unicode"Параметры обмена"),
                        Display.entry("partial_fill_enabled", unicode"Частичное исполнение разрешено"),
                        Display.entry("desc_you_send", unicode"Точная сумма, которую вы отправите из своего кошелька"),
                        Display.entry("desc_you_receive_minimum", unicode"Минимальная сумма, которую вы получите. Вы можете получить больше, если обменный курс улучшится"),
                        Display.entry("desc_recipient", unicode"Адрес кошелька, который получит обменянные токены"),
                        Display.entry("desc_swap_options", unicode"Дополнительные настройки выполнения обмена"),
                        Display.entry("desc_partial_fill", unicode"Если включено, обмен может быть частично исполнен, если невозможно выполнить полную сумму")
                    )
                )
            )
        );
    }
}
