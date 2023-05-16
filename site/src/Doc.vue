<script setup>

</script>

<template>
        <div class="px-20 pt-15 py-20 flex flex-col max-w-screen-xl xl:mx-auto">
            <h2 class="mt-4 text-4xl">
                How it works
            </h2>
            <p class="mt-4 text-base">
                After you install the extension, go to <a href="https://chat.openai.com/" class="text-blue-700 hover:text-blue-500">chat.openai.com</a>
                and find a conversation you want to share. The extension adds a "Share" button to the bottom of the conversation.
            </p>
            <figure class="max-w-lg my-4 mx-auto">
                <img src="../images/screenshot1.png" class="h-auto max-w-full border shadow" />
                <figcaption class="mt-2 text-sm text-center text-gray-500 dark:text-gray-400">Screenshot showing the new "Share" button.</figcaption>
            </figure>
            <p class="mt-4">
                After clicking "Share", a new tab will open up with your shared conversation.
            </p>
            <figure class="max-w-lg my-4 mx-auto">
                <img src="../images/screenshot2.png" class="h-auto max-w-full border shadow" />
                <figcaption class="mt-2 text-sm text-center text-gray-500 dark:text-gray-400">An example of a shared conversation.</figcaption>
            </figure>
            <p class="mt-4">
                From the shared conversation tab, you can copy the address by clicking the <img src="../images/copy.png" class="inline mx-2 rounded" /> button.
                This address can be used by anyone to view the conversation. You can paste this address to share it on social media sites, or
                scroll to the bottom of the conversation and use the social media shortcut buttons.
            </p>
            <figure class="max-w-lg my-4 mx-auto">
                <img src="../images/share.png" class="h-auto max-w-full border shadow" />
                <figcaption class="mt-2 text-sm text-center text-gray-500 dark:text-gray-400">Social sharing shortcut buttons are at the bottom of conversations.</figcaption>
            </figure>

            <h2 class="mt-4 text-4xl">
                Options
            </h2>
            <p class="mt-4 text-base">
                Once the extension is installed, click the Chrome extension puzzle piece <img src="../images/puzzle.png" class="inline mx-2 rounded" />
                and then the 
                <span class="rounded">
                    <img src="../images/logo-128.png" width="16" height="16" class="inline mx-1" />
                    ShareConversation
                </span>
                extension. This will open up a popup window with options for the extension. (You can also pin the extension
                to keep it visible without clicking on the puzzle piece).
            </p>
            <figure class="max-w-lg my-4 mx-auto">
                <img src="../images/screenshot3.png" class="h-auto max-w-full border shadow" />
                <figcaption class="mt-2 text-sm text-center text-gray-500 dark:text-gray-400">The options for sharing new conversations show up in a popup.</figcaption>
            </figure>
            <p class="mt-4">
                The options are:
                <ul class="list-disc list-inside">
                    <li class="italic pt-2">Include actual avatar picture</li>
                    <p class="ml-8 pt-2">
                        When set, the shared conversation will include your avatar image to represent your side of the conversation. The extension takes a snapshot
                        of the avatar so if your avatar later changes the shared conversation will not change.
                    </p>
                    <p class="ml-8 pt-2">
                        If not set, the shared conversation will use a solid color to represent your side of the conversation.
                    </p>
                    <li class="italic pt-2">Include in public index</li>
                    <p class="ml-8 pt-2">
                        When set, the shared conversation may be used in public lists. For example, public conversations may be featured on the ShareConversation
                        page or other places.
                    </p>
                    <p class="ml-8 pt-2">
                        When not set, only people that have the direct link to
                        the conversation will be able to see the conversation.
                        Note that if a link has been shared and then "Include in
                        public index" is turned off, the conversation will still
                        be visible to anyone that follows the link. To make the
                        conversation not visible to people with the link the
                        conversation must be deleted.
                    </p>
                    <li class="italic pt-2">Allow to be used for artificial intelligence research and development</li>
                    <p class="ml-8 pt-2">
                        When set, the data from your conversation is tagged as
                        being available for use in artificial intelligence
                        research and development. Conversation logs are a rich
                        source of data. They show what people want to chat about
                        and how people respond to different generated replies.
                        Researchers use chat logs to train and test various
                        types of language models to improve artificial
                        intelligence capabilities.
                    </p>
                    <p class="ml-8 pt-2">
                        When not set, the data from your conversation is tagged
                        as not being available for use in research and
                        development. Note that there is no actual technical
                        restriction here. It is still technically possible to
                        download your publicly shared conversations and use them
                        for purposes to which you do not consent. When this
                        option is not set data sets provided to the research
                        community in bulk form will not include your
                        conversation data.
                    </p>
                </ul>
            </p>
            <h2 class="mt-4 text-4xl">
                Managing conversations
            </h2>
            <figure class="max-w-lg my-4 mx-auto">
                <img src="../images/manage.png" class="h-auto max-w-full border shadow" />
                <figcaption class="mt-2 text-sm text-center text-gray-500 dark:text-gray-400">Your conversations can be managed here at the
                <a href="/#/manage" class="text-blue-700 hover:text-blue-500">ShareConversation.com</a> website.</figcaption>
            </figure>
            <p class="mt-4 text-base">
                If you share a conversation and then decide you want to change
                the options or delete the conversation, go to the
                <a href="/#/manage" class="">Manage</a>
                section. This is also linked from the extension popup window with a "Manage conversations" button.
            </p>
            <p class="mt-4 text-base">
                From this page you can click links to see all your shared conversations. You can toggle "public" and "research" permissions for individual
                conversations. You can download individual conversations in CSV format. You can also delete conversations.
            </p>
            <p class="mt-4 text-base">
                If you delete a conversation, it will immediately be removed from the conversation list. To undelete the conversation, click the
                "Show deleted conversations" checkbox. Once you see the deletect conversation there will be an action to undelete it.
            </p>
            <p class="mt-4 text-base">
                At the bottom of your conversations are several bulk action buttons. These buttons affect all conversations. You can turn on/off
                all the "public" or "research" settings. You can also donwload a CSV file that includes all your shared conversations.
            </p>
            <h2 class="mt-4 text-4xl">
                Note on sharing
            </h2>
            <p class="mt-4 text-base">
                If you share the same conversation multiple times you will get the same address each time. If the conversation changes and you share again, the
                extension will create a new shared conversation with the updated contents. Old shared conversations will not change. The motivation here is
                that links to shared conversations should be permanent.
            </p>

            <h2 class="mt-4 text-4xl">
                Paid members
            </h2>
            <p class="mt-4 text-base">
                Free users have a limit of 5 shared conversations. Once you reach your limit you need to upgrade your account or delete existing
                conversations. Paid users have NO LIMIT to number of conversations shared. If you try to share too many conversations, the extension
                will open the upgrade popup to prompt you to upgrade your account.
            </p>
            <p class="mt-4 text-base">
                Paid users also unlock bulk download functionality. Paid users can download all their conversations in CSV format for use in
                spreadsheets, or in JSON format for use in other programs.
            </p>
            <p class="mt-4 text-base">
                To upgrade your account, click the Chrome extension puzzle piece
                <img src="../images/puzzle.png" class="inline mx-2 rounded" />
                and then the 
                <span class="rounded">
                    <img src="../images/logo-128.png" width="16" height="16" class="inline mx-1" />
                    ShareConversation
                </span>
                extension to get the options
                popup. Then click the <img src="../images/upgrade.png"
                class="inline mx-2 rounded" /> button in the extension popup.
                For a one-time fee you can become a <span class="font-bold
                italic shadow-lg shadow-yellow-500/50">lifetime premium</span> member. You'll unlock
                the features described here and get access to new future
                features for premium members as they become available at no
                extra cost.
            </p>
            <p>
                Once you've upgraded, your account is synced with any computer and location where you use Chrome. If for any reason you need to
                manually login again, just click the same <img src="../images/upgrade.png" class="inline mx-2 rounded" /> button. From
                the payment popup window click the "Already paid? Log in" link.
            </p>
            <h2 class="mt-4 text-4xl">
                Firefox notes
            </h2>
            <p class="mt-4 text-base">
                In Firefox, the only difference in behavior is that after the extension is installed you will need to visit
                <a href="https://chat.openai.com/" class="text-blue-700 hover:text-blue-500">chat.openai.com</a>
                then click the
                <span class="rounded">
                    <img src="../images/logo-128.png" width="16" height="16" class="inline mx-1" />
                    ShareConversation
                </span> icon on the toolbar. Once you have clicked the extension icon, the "Share" button will be
                activated.
            </p>
            <figure class="max-w-lg my-4 mx-auto">
                <img src="../images/firefox_click2.png" class="h-auto max-w-full border shadow" />
                <figcaption class="mt-2 text-sm text-center text-gray-500 dark:text-gray-400">Firefox requires clicking the extension icon to give it permission
                to show the "Share" button on the ChatGPT page.</figcaption>
            </figure>
            <p class="mt-4 text-base">
                To avoid needing to click the extension to active the "Share" button, you can set optional permissions for the extension.
                Click on the extension puzzle piece
                <img src="../images/puzzle.png" class="inline mx-2 rounded" /> then choose "Manage extensions". Under "ShareConversation"
                click the "Permissions" tab and enable the optional permission "Access your data for https://chat.openai.com".
            </p>
            <figure class="max-w-lg my-4 mx-auto">
                <img src="../images/firefox_permissions2.png" class="h-auto max-w-full border shadow" />
                <figcaption class="mt-2 text-sm text-center text-gray-500 dark:text-gray-400">You can enable the optional permission "Access your data for https://chat.openai.com"
                to let the ShareConversation extension always add the "Share" button without needing the extra click.</figcaption>
            </figure>
            <h2 class="mt-4 text-4xl">
                Brave notes
            </h2>
            <p class="mt-4 text-base">
                On the Brave browser after you install the extension, to make it work you need to enable the setting "Allow Google login for extensions".
                Go to <code>brave://settings/extensions</code> and move the top toggle for "Allow Google login for extensions" to the
                right to enable. The first time you use the "Share" button to share a conversation, Brave will open a popup window
                to allow you to login to your Google account. Without changing this setting the popup would be blocked and nothing
                will happen.
            </p>
            <figure class="max-w-lg my-4 mx-auto">
                <img src="../images/brave_setup.png" class="h-auto max-w-full border shadow" />
                <figcaption class="mt-2 text-sm text-center text-gray-500 dark:text-gray-400">Brave requires enabling Google login
                for extension for the extension to work properly.</figcaption>
            </figure>
            <h2 class="mt-4 text-4xl">
                Legal
            </h2>
            <p class="mt-4 text-base">
                By installing the ShareConversation browser extension you are agreeing to the
                <router-link to="/terms_of_service.html" class="text-blue-700 hover:text-blue-500">
                    Terms of Service.
                </router-link>
            </p>
        </div>
</template>
