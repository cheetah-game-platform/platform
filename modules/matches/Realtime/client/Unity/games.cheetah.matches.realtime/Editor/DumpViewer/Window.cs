using System;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Cheetah.Matches.Realtime.Editor.DumpViewer.Provider;
using Cheetah.Matches.Realtime.Editor.DumpViewer.Sections.Objects;
using Cheetah.Matches.Realtime.Editor.DumpViewer.Sections.Users;
using Cheetah.Matches.Realtime.Editor.UIElements;
using Cheetah.Matches.Realtime.Editor.UIElements.NetworkAddress;
using Cheetah.Matches.Realtime.Editor.UIElements.RoomsSelector;
using Cheetah.Matches.Realtime.Editor.UIElements.RoomsSelector.Provider;
using Cheetah.Matches.Realtime.Editor.UIElements.StatusIndicator;
using Cheetah.Matches.Realtime.GRPC.Admin;
using Cheetah.Platform;
using Grpc.Core;
using UnityEditor;
using UnityEditor.UIElements;
using UnityEngine;
using UnityEngine.UIElements;

namespace Cheetah.Matches.Realtime.Editor.DumpViewer
{
    /// <summary>
    /// Панель просмотра сетевых команд с сервера
    /// </summary>
    public class DumpViewerWindow : EditorWindow
    {
        /// <summary>
        /// Удаленные провайдеры данных или тестовые
        /// </summary>
        private const bool RemoteProviders = true;

        private const string PathToUxml = "Packages/games.Cheetah.Matches.Realtime/Editor/DumpViewer/UI/Window.uxml";

        [MenuItem("Window/Cheetah/Dump Viewer")]
        public static void ShowWindow()
        {
            var window = GetWindow<DumpViewerWindow>();
            window.titleContent = new GUIContent("Cheetah DumpViewer");
        }

        private DumpProvider provider;
        private ObjectsViewer objectsViewer;
        private UsersViewer usersViewer;
        private StatusIndicator statusIndicator;
        private RoomsSelector roomSelector;
        private ToolbarButton dumpButton;
        private ToolbarButton exportButton;
        private ToolbarToggle objectsTab;
        private ToolbarToggle usersTab;
        private Toggle autoRefresh;
        private VisualElement tabContent;
        private ulong? selectedRoom;
        private bool connectError;
        private DumpResponse dumpResult;

        public void OnEnable()
        {
            var ui = LoadVisualTree();
            rootVisualElement.Add(ui);
            statusIndicator = ui.Q<StatusIndicator>("status");
            ConfigureConnect(ui);
            ConfigureRoomSelector(ui);
            ConfigureDumpButton(ui);
            ConfigureExportButton(ui);
            ConfigureHelpButton(ui);
            ConfigureTabs(ui);
            DisableContent();
        }

        private void ConfigureConnect(VisualElement ui)
        {
            NetworkAddress networkAddress = ui.Q<NetworkAddress>("server-address");
            networkAddress.AddConnectCallback(async address => await DoConnect(address));
        }

        private void ConfigureHelpButton(VisualElement ui)
        {
            ui.Q<ToolbarButton>("help").RegisterCallback<ClickEvent>(evt =>
            {
                Application.OpenURL("https://docs.cheetah.games/components/relay/develop/dump-panel/");
            });
        }


        private void ConfigureTabs(VisualElement ui)
        {
            objectsViewer = new ObjectsViewer(statusIndicator);
            usersViewer = new UsersViewer();
            tabContent = ui.Q<VisualElement>("tabs-content");
            autoRefresh = ui.Q<Toggle>("auto-refresh");
            var tabsController = new TabsController();
            objectsTab = ui.Q<ToolbarToggle>("objectsTab");
            tabsController.RegisterTab(objectsTab, () => { SwitchContentTo(tabContent, objectsViewer); });
            usersTab = ui.Q<ToolbarToggle>("usersTab");
            tabsController.RegisterTab(usersTab, () => { SwitchContentTo(tabContent, usersViewer); });
            tabsController.SwitchToFirst();
        }

        private void ConfigureDumpButton(VisualElement ui)
        {
            dumpButton = ui.Q<ToolbarButton>("dump-button");
            dumpButton.SetEnabled(false);
            dumpButton.RegisterCallback<ClickEvent>(OnDumpButtonPressed);
        }

        private void ConfigureRoomSelector(VisualElement ui)
        {
            roomSelector = ui.Q<RoomsSelector>("room-selector");
            roomSelector.SetEnabled(false);
            roomSelector.RoomSelectEvent += RoomRoomSelect;
            roomSelector.RoomUnselectEvent += RoomUnselect;
        }

        private void RoomUnselect()
        {
            selectedRoom = null;
            dumpButton.SetEnabled(false);
        }

        private Task RoomRoomSelect(ulong room)
        {
            selectedRoom = room;
            dumpButton.SetEnabled(!connectError);
            return Task.CompletedTask;
        }

        private async void OnDumpButtonPressed(ClickEvent evt)
        {
            DisableContent();
            try
            {
                var first = true;
                while (autoRefresh.value || first)
                {
                    statusIndicator.ResetStatus();
                    var currentDumpResult = await provider.Dump((ulong)selectedRoom);
                    objectsViewer.SetData(currentDumpResult);
                    usersViewer.SetData(currentDumpResult);
                    dumpResult = currentDumpResult;
                    await Task.Delay(60);
                    first = false;
                }

                EnableContent();
            }
            catch (RpcException e)
            {
                var emptyDump = new DumpResponse();
                objectsViewer.SetData(emptyDump);
                usersViewer.SetData(emptyDump);
                Debug.LogError(e);
                statusIndicator.SetStatus("Get dump error  " + e.Status.Detail, StatusIndicator.MessageType.Error);
            }
        }

        private void EnableContent()
        {
            dumpButton.SetEnabled(true);
        }

        private void DisableContent()
        {
            dumpButton.SetEnabled(false);
        }


        private static void SwitchContentTo(VisualElement content, VisualElement element)
        {
            content.Clear();
            content.Add(element);
        }


        private static TemplateContainer LoadVisualTree()
        {
            var uiAsset = AssetDatabase.LoadAssetAtPath<VisualTreeAsset>(PathToUxml);
            return uiAsset.CloneTree();
        }


        private async void Update()
        {
            if (provider == null)
            {
                return;
            }

            try
            {
                await roomSelector.Update();
                await Task.Delay(TimeSpan.FromSeconds(5));
            }
            catch (RpcException e)
            {
                ConnectErrorState(e);
            }
        }

        private void ConnectedState()
        {
            connectError = false;
            dumpButton.SetEnabled(selectedRoom != null);
            statusIndicator.SetStatus("Connected", StatusIndicator.MessageType.Regular);
        }

        private void ConnectErrorState(Exception exception)
        {
            Debug.LogError(exception);
            connectError = true;
            provider = null;
            roomSelector.RemoveProvider();
            dumpButton.SetEnabled(false);
            var message = exception.Message;
            if (exception is RpcException rpcException)
            {
                message = "grpc status code: " + rpcException.Status.StatusCode.ToString();
            }

            statusIndicator.SetStatus("Cannot connect to server: " + message, StatusIndicator.MessageType.Error);
        }


        private async void OnDestroy()
        {
            if (provider != null)
            {
                await provider.Destroy();
            }
        }

        private void ConfigureExportButton(VisualElement ui)
        {
            exportButton = ui.Q<ToolbarButton>("export");
            exportButton.RegisterCallback<ClickEvent>(_ =>
            {
                if (dumpResult != null)
                {
                    var path = EditorUtility.SaveFolderPanel(
                        "Select folders for store dumps (objects.csv, users.csv)",
                        "",
                        "");
                    if (path.Length == 0) return;


                    var objects = new StringBuilder();
                    objects.Append("id").Append("\t");
                    objects.Append("template").Append("\t");
                    objects.Append("ownerUserId").Append("\t");
                    objects.Append("groups").Append("\t");
                    objects.Append("created").Append("\t");
                    objects.Append("fields").Append("\t");
                    objects.Append("compareAndSetOwners");
                    objects.AppendLine();
                    foreach (var dumpObject in dumpResult.Objects)
                    {
                        objects.Append(dumpObject.Id).Append("\t");
                        objects.Append(dumpObject.Template).Append("\t");
                        objects.Append(dumpObject.HasOwnerUserId ? dumpObject.OwnerUserId : "None").Append("\t");
                        objects.Append(dumpObject.Groups).Append("\t");
                        objects.Append(dumpObject.Created).Append("\t");
                        objects.Append(string.Join(",", dumpObject.Fields.Select(p => p.Id + "=" + p.Value))).Append("\t");
                        objects.Append(string.Join(",", dumpObject.CompareAndSetOwners.Select(p => p.Key + "=" + p.Value)));
                        objects.AppendLine();
                    }

                    var users = new StringBuilder();
                    users.Append("id").Append("\t");
                    users.Append("groups").Append("\t");
                    users.Append("attached").Append("\t");
                    users.Append("compareAndSetCleaners");
                    users.AppendLine();
                    foreach (var dumpUser in dumpResult.Users)
                    {
                        users.Append(dumpUser.Id).Append("\t");
                        users.Append(dumpUser.Groups).Append("\t");
                        users.Append(dumpUser.Attached).Append("\t");
                        users.Append(string.Join(",",
                            dumpUser.CompareAndSetCleaners.Select(p => (p.GameObjectId, p.GameObjectOwnerUser, p.FieldId, p.Value)))).Append("\t");
                        users.AppendLine();
                    }

                    File.WriteAllText(path + "/objects.csv", objects.ToString());
                    File.WriteAllText(path + "/users.csv", users.ToString());
                }
            });
        }

        private async Task DoConnect(string address)
        {
            try
            {
                var uri = new Uri(address);
                var clusterConnector = new ClusterConnector(uri.Host, uri.Port, uri.Scheme == "https");
                var roomsProvider = RemoteProviders
                    ? (RoomsProvider)new RemoteRoomsProvider(clusterConnector)
                    : new TestRoomsProvider();

                var rooms = await roomsProvider.GetRooms();
                roomSelector.SetProvider(roomsProvider);
                provider = RemoteProviders
                    ? (DumpProvider)new RemoteDumpProvider(clusterConnector)
                    : new TestDumpProvider();
                ConnectedState();
            }
            catch (Exception e)
            {
                ConnectErrorState(e);
            }
        }
    }
}