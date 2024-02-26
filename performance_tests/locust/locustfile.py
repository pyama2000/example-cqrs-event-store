from locust import HttpUser, task


class Widget(HttpUser):
    widget_id = ""

    def on_start(self):
        response = self.client.post(
            "/widgets",
            json={"widget_name": "部品名", "widget_description": "部品の説明"},
        )
        self.widget_id = response.json()["widget_id"]

    @task
    def create_widget(self):
        self.client.post(
            "/widgets",
            json={"widget_name": "部品名", "widget_description": "部品の説明"},
        )

    @task
    def change_widget_name(self):
        self.client.post(
            f"/widgets/{self.widget_id}/name",
            # "/widgets/01HQ24H0FTENXF52E1BEMH9R9E/name",
            json={"widget_name": "部品名"},
        )

    @task
    def change_widget_description(self):
        self.client.post(
            f"/widgets/{self.widget_id}/description",
            # "/widgets/01HQ24H0FTENXF52E1BEMH9R9E/description",
            json={"widget_description": "部品名"},
        )
