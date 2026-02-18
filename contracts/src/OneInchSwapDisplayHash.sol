// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

import "./Display.sol";

library OneInchSwapDisplayHash {

    bytes32 public constant ONE_INCH_SWAP_DISPLAY_HASH = keccak256(
        abi.encode(
            DISPLAY_TH,
            keccak256(bytes("function swap(address executor, (address srcToken, address dstToken, address srcReceiver, address dstReceiver, uint256 amount, uint256 minReturnAmount, uint256 flags) desc, bytes data)")),
            keccak256(bytes("$labels.swap_title")),
            keccak256(bytes("$labels.swap_description")),
            keccak256(abi.encodePacked(
                // Field 1: You Send - tokenAmountField
                keccak256(abi.encode(
                    FIELD_TH,
                    keccak256(bytes("$labels.you_send")),
                    keccak256(bytes("$labels.desc_you_send")),
                    keccak256(bytes("tokenAmount")),
                    keccak256(bytes("")), // checks
                    keccak256(abi.encodePacked(
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("token")), keccak256(bytes("$data.desc.srcToken")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("amount")), keccak256(bytes("$data.desc.amount"))))
                    )),
                    keccak256(bytes("")) // fields
                )),
                // Field 2: You Receive (Minimum) - tokenAmountField
                keccak256(abi.encode(
                    FIELD_TH,
                    keccak256(bytes("$labels.you_receive_minimum")),
                    keccak256(bytes("$labels.desc_you_receive_minimum")),
                    keccak256(bytes("tokenAmount")),
                    keccak256(bytes("")), // checks
                    keccak256(abi.encodePacked(
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("token")), keccak256(bytes("$data.desc.dstToken")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("amount")), keccak256(bytes("$data.desc.minReturnAmount"))))
                    )),
                    keccak256(bytes("")) // fields
                )),
                // Field 3: Recipient - addressField
                keccak256(abi.encode(
                    FIELD_TH,
                    keccak256(bytes("$labels.recipient")),
                    keccak256(bytes("$labels.desc_recipient")),
                    keccak256(bytes("address")),
                    keccak256(bytes("")), // checks
                    keccak256(abi.encodePacked(
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("value")), keccak256(bytes("$data.desc.dstReceiver"))))
                    )),
                    keccak256(bytes("")) // fields
                )),
                // Field 4: Swap Options - bitmaskField
                keccak256(abi.encode(
                    FIELD_TH,
                    keccak256(bytes("$labels.swap_options")),
                    keccak256(bytes("$labels.desc_swap_options")),
                    keccak256(bytes("bitmask")),
                    keccak256(bytes("")), // checks
                    keccak256(abi.encodePacked(
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("value")), keccak256(bytes("$data.desc.flags")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("#0")), keccak256(bytes("$labels.partial_fill_enabled"))))
                    )),
                    keccak256(bytes("")) // fields
                ))
            )),
            keccak256(abi.encodePacked(
                // Labels for "en"
                keccak256(abi.encode(
                    LABELS_TH,
                    keccak256(bytes("en")),
                    keccak256(abi.encodePacked(
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("swap_title")), keccak256(bytes("1inch Swap")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("swap_description")), keccak256(bytes("Exchange one token for another using the 1inch aggregation protocol")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("you_send")), keccak256(bytes("You Send")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("you_receive_minimum")), keccak256(bytes("You Receive (Minimum)")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("recipient")), keccak256(bytes("Recipient")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("swap_options")), keccak256(bytes("Swap Options")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("partial_fill_enabled")), keccak256(bytes("Partial fill allowed")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("desc_you_send")), keccak256(bytes("The exact amount you will send from your wallet")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("desc_you_receive_minimum")), keccak256(bytes("The minimum amount you will receive. You may receive more if the exchange rate improves")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("desc_recipient")), keccak256(bytes("The wallet address that will receive the swapped tokens")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("desc_swap_options")), keccak256(bytes("Additional swap execution settings")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("desc_partial_fill")), keccak256(bytes("If enabled, the swap can be partially filled if the full amount cannot be executed"))))
                    ))
                )),
                // Labels for "es"
                keccak256(abi.encode(
                    LABELS_TH,
                    keccak256(bytes("es")),
                    keccak256(abi.encodePacked(
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("swap_title")), keccak256(bytes(unicode"Intercambio 1inch")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("swap_description")), keccak256(bytes(unicode"Intercambia un token por otro usando el protocolo de agregación 1inch")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("you_send")), keccak256(bytes(unicode"Envías")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("you_receive_minimum")), keccak256(bytes(unicode"Recibes (Mínimo)")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("recipient")), keccak256(bytes(unicode"Destinatario")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("swap_options")), keccak256(bytes(unicode"Opciones de Intercambio")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("partial_fill_enabled")), keccak256(bytes(unicode"Llenado parcial permitido")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("desc_you_send")), keccak256(bytes(unicode"La cantidad exacta que enviarás desde tu billetera")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("desc_you_receive_minimum")), keccak256(bytes(unicode"La cantidad mínima que recibirás. Puedes recibir más si mejora el tipo de cambio")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("desc_recipient")), keccak256(bytes(unicode"La dirección de billetera que recibirá los tokens intercambiados")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("desc_swap_options")), keccak256(bytes(unicode"Configuraciones adicionales de ejecución del intercambio")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("desc_partial_fill")), keccak256(bytes(unicode"Si está habilitado, el intercambio puede llenarse parcialmente si no se puede ejecutar la cantidad completa"))))
                    ))
                )),
                // Labels for "ru"
                keccak256(abi.encode(
                    LABELS_TH,
                    keccak256(bytes("ru")),
                    keccak256(abi.encodePacked(
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("swap_title")), keccak256(bytes(unicode"Обмен 1inch")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("swap_description")), keccak256(bytes(unicode"Обменяйте один токен на другой, используя протокол агрегации 1inch")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("you_send")), keccak256(bytes(unicode"Вы отправляете")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("you_receive_minimum")), keccak256(bytes(unicode"Вы получаете (Минимум)")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("recipient")), keccak256(bytes(unicode"Получатель")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("swap_options")), keccak256(bytes(unicode"Параметры обмена")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("partial_fill_enabled")), keccak256(bytes(unicode"Частичное исполнение разрешено")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("desc_you_send")), keccak256(bytes(unicode"Точная сумма, которую вы отправите из своего кошелька")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("desc_you_receive_minimum")), keccak256(bytes(unicode"Минимальная сумма, которую вы получите. Вы можете получить больше, если обменный курс улучшится")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("desc_recipient")), keccak256(bytes(unicode"Адрес кошелька, который получит обменянные токены")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("desc_swap_options")), keccak256(bytes(unicode"Дополнительные настройки выполнения обмена")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("desc_partial_fill")), keccak256(bytes(unicode"Если включено, обмен может быть частично исполнен, если невозможно выполнить полную сумму"))))
                    ))
                ))
            ))
        )
    );

}
