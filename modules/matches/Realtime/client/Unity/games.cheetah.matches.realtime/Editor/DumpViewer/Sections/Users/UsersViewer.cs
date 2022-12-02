using System.Collections;
using System.Collections.Generic;
using System.Linq;
using Cheetah.Matches.Realtime.Editor.UIElements.Table;
using Cheetah.Matches.Realtime.GRPC.Admin;
using UnityEditor;
using UnityEngine.UIElements;

namespace Cheetah.Matches.Realtime.Editor.DumpViewer.Sections.Users
{
    public class UsersViewer : VisualElement
    {
        private TableElement usersTable;
        private TableElement createdObjectsTable;
        private TableElement objectsInUserGroupTable;
        private DumpResponse dumpResponse;
        private readonly Label selectedUserIdLabel;


        public UsersViewer()
        {
            var uiAsset =
                AssetDatabase.LoadAssetAtPath<VisualTreeAsset>("Packages/games.Cheetah.Matches.Realtime/Editor/DumpViewer/Sections/Users/Panel.uxml");
            uiAsset.CloneTree(this);

            selectedUserIdLabel = this.Q<Label>("item-user-id");
            ConfigureUsersTable();
            ConfigureCreateObjectTable();
            ConfigureObjectsInUserGroups();
        }

        private void ConfigureUsersTable()
        {
            usersTable = this.Q<TableElement>("users");
            TablesConfigurator.ConfigureUsersTable(usersTable);
            usersTable.RegisterSelectedListener(OnUserSelect);
        }

        private void OnUserSelect(IEnumerable<object> obj)
        {
            if (!obj.Any())
            {
                ResetSelectedUser();
                return;
            }

            var user = (DumpUser)obj.First();
            selectedUserIdLabel.text = user.Id.ToString();
            createdObjectsTable.SetData(dumpResponse.Objects.Where(o=>o.OwnerUserId==user.Id).ToList());
            objectsInUserGroupTable.SetData(dumpResponse.Objects.Where(o => (o.Groups & user.Groups)>0).ToList());
        }

        private void ResetSelectedUser()
        {
            createdObjectsTable.SetData(new ArrayList());
            objectsInUserGroupTable.SetData(new ArrayList());
            selectedUserIdLabel.text = "unselect";
        }

        private void ConfigureCreateObjectTable()
        {
            createdObjectsTable = this.Q<TableElement>("created-objects");
            TablesConfigurator.ConfigureObjectsTable(createdObjectsTable);
        }

        private void ConfigureObjectsInUserGroups()
        {
            objectsInUserGroupTable = this.Q<TableElement>("groups-objects");
            TablesConfigurator.ConfigureObjectsTable(objectsInUserGroupTable);
        }

        public void SetData(DumpResponse dumpResponse)
        {
            this.dumpResponse = dumpResponse;
            usersTable.SetData(dumpResponse.Users);
        }
    }
}