using System;
using System.Collections.Generic;
using System.Linq;
using Cheetah.Matches.Realtime.GRPC.Admin;
using UnityEditor;

namespace Cheetah.Matches.Realtime.Editor.NetworkCommandsViewer.UI
{
    /// <summary>
    /// Список столбцов таблицы, управление видимостью.
    /// </summary>
    public class Columns
    {
        public delegate void Update();

        public event Update OnActiveColumnsUpdate;

        private readonly IDictionary<string, bool> enabledCache = new Dictionary<string, bool>();


        public readonly IList<Column> AllColumns = new List<Column>()
        {
            new Column("Time", 110, 110, null, (command) =>
            {
                var time = TimeSpan.FromSeconds(command.Time);
                var dateTime = new DateTime(time.Ticks);
                var localTime = dateTime.ToLocalTime();
                return localTime.Hour.ToString("D2") + ":" + localTime.Minute.ToString("D2") + ":" + localTime.Second.ToString("D2") + "." +
                       localTime.Millisecond.ToString("D3");
            }),
            new Column("Direction", 110, 110, null, (command) => command.Direction),
            new Column("User", 70, 70, null, (command) => command.UserId.ToString()),
            new Column("Template", 150, 150, null, (command) =>
            {
                if (!command.HasTemplate)
                {
                    return "unknown";
                }

                return command.Template.ToString();
            }),
            new Column("Field", 150, 150, 1, (command) =>
            {
                if (!command.HasFieldType || !command.HasFieldId)
                {
                    return "";
                }

                return command.FieldId.ToString();
            }),
            new Column("Object", 200, null, null, (command) => command.ObjectId),
            new Column("Command", 200, null, null, (command) => command.Command_),
            new Column("Value", 250, null, 1, (command) => command.Value),
        };

        public bool IsEnable(Column column)
        {
            if (enabledCache.TryGetValue(column.header, out var isCacheEnable))
            {
                return isCacheEnable;
            }

            var isEnable = EditorPrefs.GetBool(GetKey(column), true);
            enabledCache[column.header] = isEnable;
            return isEnable;
        }

        public void SetEnable(Column column, bool isEnable)
        {
            EditorPrefs.SetBool(GetKey(column), isEnable);
            enabledCache[column.header] = isEnable;
            OnActiveColumnsUpdate?.Invoke();
        }

        private static string GetKey(Column column)
        {
            return "cheetah_relay_network_commands_columns_" + column.header;
        }

        public List<Column> GetEnabledColumns()
        {
            return AllColumns.Where(IsEnable).ToList();
        }
    }

    public class Column
    {
        public delegate string Converter(Command command);

        public readonly string header;
        public readonly int? minWidth;
        public readonly int? maxWidth;
        public readonly float? flexGrow;
        private readonly Converter converter;

        public Column(string header, int? minWidth, int? maxWidth, float? flexGrow, Converter converter)
        {
            this.header = header;
            this.minWidth = minWidth;
            this.maxWidth = maxWidth;
            this.flexGrow = flexGrow;
            this.converter = converter;
        }

        public string GetValue(Command command)
        {
            return converter.Invoke(command);
        }
    }
}