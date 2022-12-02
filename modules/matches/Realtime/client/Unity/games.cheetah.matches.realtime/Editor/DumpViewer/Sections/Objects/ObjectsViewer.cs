using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Cheetah.Matches.Realtime.Editor.DumpViewer.Sections.Objects.Query;
using Cheetah.Matches.Realtime.Editor.DumpViewer.TypesExtension;
using Cheetah.Matches.Realtime.Editor.UIElements.HistoryTextField;
using Cheetah.Matches.Realtime.Editor.UIElements.StatusIndicator;
using Cheetah.Matches.Realtime.Editor.UIElements.Table;
using Cheetah.Matches.Realtime.GRPC.Admin;
using Cheetah.Matches.Realtime.GRPC.Shared;
using UnityEditor;
using UnityEngine.UIElements;

namespace Cheetah.Matches.Realtime.Editor.DumpViewer.Sections.Objects
{
    public class ObjectsViewer : VisualElement
    {
        private readonly StatusIndicator statusIndicator;
        private TableElement objectsTable;
        private TableElement fieldsTable;
        private TableElement loadUsersTable;
        private DumpResponse dumpResponse;
        private Rule rule;
        private Label itemObjectId;
        

        /// <summary>
        /// Описание поля объекта
        /// </summary>
        public class FieldItem
        {
            public uint Id;
            public FieldValue Value;
        }

        public ObjectsViewer(StatusIndicator statusIndicator)
        {
            this.statusIndicator = statusIndicator;
            var uiAsset =
                AssetDatabase.LoadAssetAtPath<VisualTreeAsset>("Packages/games.Cheetah.Matches.Realtime/Editor/DumpViewer/Sections/Objects/Panel.uxml");
            uiAsset.CloneTree(this);


            ConfigureLoadToUsersTable();
            ConfigureObjectInfo();
            ConfigureQueryField();
            ConfigureObjectsTable();
            ConfigureFieldsTable();
        }

        private void ConfigureLoadToUsersTable()
        {
            loadUsersTable = this.Q<TableElement>("users");
            TablesConfigurator.ConfigureUsersTable(loadUsersTable);
        }

        private void ConfigureObjectInfo()
        {
            itemObjectId = this.Q<Label>("item-object-id");
        }

        private void ConfigureQueryField()
        {
            var queryField = this.Q<HistoryTextField>("query");
            queryField.RegisterOnChangeListener(ApplyQuery);
        }

        private Task ApplyQuery(string value)
        {
            value = value.Trim();
            try
            {
                rule = value.Length == 0 ? null : Parser.Parse(value);
                UpdateDataWithFilter();
                statusIndicator.ResetStatus();
            }
            catch (ParserException e)
            {
                statusIndicator.SetStatus(e.Message, StatusIndicator.MessageType.Error);
            }
            
            return Task.CompletedTask;
        }

        private void ConfigureFieldsTable()
        {
            fieldsTable = this.Q<TableElement>("fields");
            fieldsTable.AddColumn("Тип", 70, null, null, o =>
            {
                var item = (FieldItem)o;
                return ToFieldType(item.Value.VariantCase).ToString();
            });
            fieldsTable.AddColumn("Поле", 200, null, 0, o =>
            {
                var item = (FieldItem)o;
                return item.Id.ToString();
            });
            fieldsTable.AddColumn("Значение", 200, null, 1, o =>
            {
                var item = (FieldItem)o;
                return item.Value.VariantCase switch
                {
                    FieldValue.VariantOneofCase.Long => item.Value.Long.ToString(),
                    FieldValue.VariantOneofCase.Double => item.Value.Double.ToString(),
                    FieldValue.VariantOneofCase.Structure => string.Join(",", item.Value.Structure.ToByteArray()),
                    _ => throw new ArgumentOutOfRangeException()
                };
            });
            
            fieldsTable.SetOrderComparer(new FieldItemIdComparator());
        }

        private void ConfigureObjectsTable()
        {
            objectsTable = this.Q<TableElement>("objects");
            TablesConfigurator.ConfigureObjectsTable(objectsTable);
            objectsTable.RegisterSelectedListener(OnObjectSelect);
        }

        private void OnObjectSelect(IEnumerable<object> items)
        {
            if (!items.Any())
            {
                ResetSelectedObject();
                return;
            }

            var item = items.First();
            var obj = (DumpObject)item;

            ShowObjectInfo(obj);
            ShowFieldsForObject(obj);

            var loadedUsers = dumpResponse.Users.Where(u => (u.Groups & obj.Groups) != 0).ToList();
            loadUsersTable.SetData(loadedUsers);
        }

        private void ResetSelectedObject()
        {
            loadUsersTable.SetData(new ArrayList());
            fieldsTable.SetData(new ArrayList());
            itemObjectId.text = "unselect";
        }

        private void ShowObjectInfo(DumpObject obj)
        {
            itemObjectId.text = TablesConfigurator.GetDumpObjectId(obj);
        }

        private void ShowFieldsForObject(DumpObject obj)
        {
            var fields = obj.Fields.Select(pair => new FieldItem { Id = pair.Id, Value = pair.Value}).ToList();
            fields = rule != null ? fields.FindAll(c => rule.Filter(c)).ToList() : fields;
            fieldsTable.SetData(fields);
        }

        public void SetData(DumpResponse data)
        {
            dumpResponse = data;
            UpdateDataWithFilter();
        }


        private void UpdateDataWithFilter()
        {
            var items = rule != null ? dumpResponse.Objects.Where(c => rule.Filter(c)).ToList() : dumpResponse.Objects.ToList();
            objectsTable.SetData(items);
        }
        
        private static FieldType ToFieldType(FieldValue.VariantOneofCase valueType)
        {
            return valueType switch
            {
                FieldValue.VariantOneofCase.Long => FieldType.Long,
                FieldValue.VariantOneofCase.Double => FieldType.Double,
                FieldValue.VariantOneofCase.Structure => FieldType.Structure,
                _ => throw new ArgumentOutOfRangeException(nameof(valueType), valueType, null)
            };
        }
    }
}